use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::error::{Result, SteamError};
use crate::error_code::ErrorCode;

/// ! Bumped whenever the table layout changes. An older database is upgraded
/// ! in place by `UPGRADES`; one written by a newer build is refused, never
/// ! replaced, because it may hold Steam Guard secrets that exist nowhere else.
pub const SCHEMA_VERSION: i64 = 2;

const SCHEMA: &str = "
CREATE TABLE setting (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- The whole vault payload: encrypted as one blob, or plain JSON when no passphrase is set.
CREATE TABLE vault (
    id   INTEGER PRIMARY KEY CHECK (id = 1),
    body TEXT NOT NULL
) STRICT;

-- Public community profile shown on the cards, also while the vault is locked.
CREATE TABLE profile (
    steam_id     TEXT PRIMARY KEY,
    persona_name TEXT NOT NULL,
    avatar       BLOB,
    avatar_mime  TEXT,
    fetched_at   INTEGER NOT NULL
) STRICT;

CREATE TABLE folder (
    id       TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name     TEXT NOT NULL,
    color    TEXT,
    icon     TEXT
) STRICT;

CREATE TABLE account_folder (
    steam_id  TEXT PRIMARY KEY,
    folder_id TEXT NOT NULL REFERENCES folder(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE account_color (
    steam_id TEXT PRIMARY KEY,
    color    TEXT NOT NULL
) STRICT;
";

/// ! `UPGRADES[n - 1]` takes a database from version `n` to `n + 1`. Each step
/// ! only adds: a 0.1.0 database may hold the only copy of an authenticator, so
/// ! nothing is dropped or rewritten, and `IF NOT EXISTS` keeps a step
/// ! re-runnable after an interrupted upgrade.
const UPGRADES: [&str; 1] = ["
CREATE TABLE IF NOT EXISTS folder (
    id       TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name     TEXT NOT NULL,
    color    TEXT,
    icon     TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS account_folder (
    steam_id  TEXT PRIMARY KEY,
    folder_id TEXT NOT NULL REFERENCES folder(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS account_color (
    steam_id TEXT PRIMARY KEY,
    color    TEXT NOT NULL
) STRICT;
"];

/// Cloning shares the same connection.
#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Database")
            .field("path", &self.path)
            .finish()
    }
}

/// `moved_aside` is set when a corrupt file was found and kept next to the
/// fresh database under a name that is never reused.
pub struct Opened {
    pub database: Database,
    pub moved_aside: Option<PathBuf>,
}

/// ! Only corruption justifies moving the file aside: a locked file, a
/// ! permission error or a schema from another version must leave the data
/// ! (which may hold Steam Guard secrets) exactly where it is.
#[derive(Debug)]
enum OpenFailure {
    /// Not a SQLite file, or one that fails its integrity check.
    Corrupt(String),
    /// Written by a build with a newer `SCHEMA_VERSION`.
    Version(i64),
    /// Locked, unreadable, out of disk space, and so on.
    Other(rusqlite::Error),
}

impl OpenFailure {
    fn classify(error: rusqlite::Error) -> Self {
        match &error {
            rusqlite::Error::SqliteFailure(failure, _)
                if matches!(
                    failure.code,
                    rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
                ) =>
            {
                OpenFailure::Corrupt(error.to_string())
            }
            _ => OpenFailure::Other(error),
        }
    }

    fn into_error(self, context: &'static str) -> SteamError {
        match self {
            OpenFailure::Version(found) => SteamError::Io {
                context: format!("{context}: schema version {found}, expected {SCHEMA_VERSION}"),
                source: io::Error::other(ErrorCode::DatabaseVersionMismatch.as_str()),
            },
            OpenFailure::Corrupt(message) => SteamError::io(context, io::Error::other(message)),
            OpenFailure::Other(error) => SteamError::io(context, error_as_io(&error)),
        }
    }
}

impl Database {
    pub fn open(path: &Path) -> Result<Opened> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SteamError::io("Create the application data folder", error))?;
        }
        match Self::connect(path) {
            Ok(connection) => Ok(Opened {
                database: Self::wrap(connection, path),
                moved_aside: None,
            }),
            Err(OpenFailure::Corrupt(_)) if path.exists() => {
                let backup = unused_backup_path(path);
                move_aside(path, &backup)?;
                let connection = Self::connect(path)
                    .map_err(|error| error.into_error("Create the application database"))?;
                Ok(Opened {
                    database: Self::wrap(connection, path),
                    moved_aside: Some(backup),
                })
            }
            Err(error) => Err(error.into_error("Open the application database")),
        }
    }

    pub fn in_memory() -> Result<Self> {
        let connection = Connection::open_in_memory().map_err(|error| {
            SteamError::io("Create the application database", error_as_io(&error))
        })?;
        Ok(Self::wrap(
            Self::prepare(connection)
                .map_err(|error| error.into_error("Create the application database"))?,
            PathBuf::from(":memory:"),
        ))
    }

    pub fn transaction<T>(
        &self,
        context: &'static str,
        work: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let mut connection = self.lock();
        // Taking the write lock up front avoids a failed upgrade when a save reads before it writes.
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        let value =
            work(&transaction).map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        transaction
            .commit()
            .map_err(|error| SteamError::io(context, error_as_io(&error)))?;
        Ok(value)
    }

    pub fn read<T>(
        &self,
        context: &'static str,
        work: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let connection = self.lock();
        work(&connection).map_err(|error| SteamError::io(context, error_as_io(&error)))
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn wrap(connection: Connection, path: impl Into<PathBuf>) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
            path: path.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn connect(path: &Path) -> std::result::Result<Connection, OpenFailure> {
        let connection = Connection::open(path).map_err(OpenFailure::classify)?;
        restrict_permissions(path);
        Self::prepare(connection)
    }

    fn prepare(mut connection: Connection) -> std::result::Result<Connection, OpenFailure> {
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(OpenFailure::classify)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(OpenFailure::classify)?;
        // ! The integrity check runs first: a damaged file must be recognised as
        // ! such rather than reported as a version or query error.
        let check: String = connection
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .map_err(OpenFailure::classify)?;
        if check != "ok" {
            return Err(OpenFailure::Corrupt(format!("quick_check: {check}")));
        }
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(OpenFailure::classify)?;
        if version == 0 {
            connection
                .execute_batch(SCHEMA)
                .map_err(OpenFailure::classify)?;
            connection
                .pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(OpenFailure::classify)?;
        } else if version < SCHEMA_VERSION {
            Self::upgrade(&mut connection, version)?;
        } else if version != SCHEMA_VERSION {
            return Err(OpenFailure::Version(version));
        }
        Ok(connection)
    }

    /// Each step and its version bump commit together, so a crash midway
    /// leaves the file at the version it reports.
    fn upgrade(connection: &mut Connection, from: i64) -> std::result::Result<(), OpenFailure> {
        for (index, step) in UPGRADES.iter().enumerate() {
            let target = index as i64 + 2;
            if target <= from {
                continue;
            }
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(OpenFailure::classify)?;
            transaction
                .execute_batch(step)
                .map_err(OpenFailure::classify)?;
            transaction
                .pragma_update(None, "user_version", target)
                .map_err(OpenFailure::classify)?;
            transaction.commit().map_err(OpenFailure::classify)?;
        }
        Ok(())
    }
}

/// `<name>.db.bak`, or `<name>.db.<unix time>[-n].bak` when that exists already:
/// a backup made by an earlier recovery is never overwritten.
fn unused_backup_path(path: &Path) -> PathBuf {
    let first = path.with_extension("db.bak");
    if !first.exists() {
        return first;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let mut attempt = 0u32;
    loop {
        let suffix = if attempt == 0 {
            format!("db.{stamp}.bak")
        } else {
            format!("db.{stamp}-{attempt}.bak")
        };
        let candidate = path.with_extension(suffix);
        if !candidate.exists() {
            return candidate;
        }
        attempt += 1;
    }
}

/// Moves the database and the journal files SQLite keeps next to it, so the
/// fresh database does not pick up a write-ahead log from the damaged one.
fn move_aside(path: &Path, backup: &Path) -> Result<()> {
    fs::rename(path, backup)
        .map_err(|error| SteamError::io("Keep the unreadable database aside", error))?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let sibling = PathBuf::from(format!("{}{suffix}", path.display()));
        if sibling.exists() {
            let target = PathBuf::from(format!("{}{suffix}", backup.display()));
            let _ = fs::rename(&sibling, &target);
        }
    }
    Ok(())
}

/// ! The database holds Steam Guard secrets, so only the owner may read it.
/// ! SQLite's journal files copy the database's permissions.
fn restrict_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

fn error_as_io(error: &rusqlite::Error) -> std::io::Error {
    std::io::Error::other(error.to_string())
}
