use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::error::{Result, SteamError};
use crate::error_code::ErrorCode;

// ! Bumped when the table layout changes; a newer-than-current database is refused, never overwritten, since it may hold secrets nowhere else.
pub const SCHEMA_VERSION: i64 = 5;

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

-- Write-ahead record for a link/finalize/transfer step: written before the
-- Steam call, kept until the resulting account is durably saved in `vault`.
-- `accepted` only turns on after Steam has confirmed the step; `open` uses it
-- to tell an aborted attempt (unsafe to recover: Steam's answer is unknown)
-- from one Steam already accepted (recoverable from this row alone).
CREATE TABLE link_wal (
    steam_id  TEXT PRIMARY KEY,
    encrypted INTEGER NOT NULL,
    nonce     TEXT,
    body      TEXT NOT NULL,
    accepted  INTEGER NOT NULL
) STRICT;
";

// ! Each step only adds columns/tables so a 0.1.0 database is never dropped or rewritten, and is safely re-runnable after an interrupted upgrade.
const UPGRADES: [&str; 4] = [
    "
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
",
    // ! No table change: version 3 only turns `secure_delete` on. Reaching it
    // through an upgrade means the file was written before that pragma
    // existed, so its free pages may still hold plaintext secrets; `open`
    // runs one `VACUUM` right after this step commits to rewrite them.
    "",
    "
CREATE TABLE IF NOT EXISTS link_wal (
    steam_id  TEXT PRIMARY KEY,
    encrypted INTEGER NOT NULL,
    nonce     TEXT,
    body      TEXT NOT NULL,
    accepted  INTEGER NOT NULL
) STRICT;
",
    // ! Version 5: preference enums move from lower-case wire tokens to their Rust variant names (SCREAMING_SNAKE_CASE).
    "
UPDATE setting SET value = CASE value
    WHEN '\"never\"' THEN '\"NEVER\"'
    WHEN '\"startup\"' THEN '\"STARTUP\"'
    WHEN '\"15m\"' THEN '\"FIFTEEN_MINUTES\"'
    WHEN '\"30m\"' THEN '\"THIRTY_MINUTES\"'
    WHEN '\"1h\"' THEN '\"ONE_HOUR\"'
    WHEN '\"6h\"' THEN '\"SIX_HOURS\"'
    WHEN '\"1d\"' THEN '\"ONE_DAY\"'
    WHEN '\"1w\"' THEN '\"ONE_WEEK\"'
    WHEN '\"display_name\"' THEN '\"DISPLAY_NAME\"'
    WHEN '\"identifier\"' THEN '\"IDENTIFIER\"'
    WHEN '\"both\"' THEN '\"BOTH\"'
    WHEN '\"5m\"' THEN '\"FIVE_MINUTES\"'
    WHEN '\"10m\"' THEN '\"TEN_MINUTES\"'
    WHEN '\"20m\"' THEN '\"TWENTY_MINUTES\"'
    WHEN '\"2h\"' THEN '\"TWO_HOURS\"'
    WHEN '\"4h\"' THEN '\"FOUR_HOURS\"'
    ELSE value
END
WHERE key IN ('update_interval', 'search_scope', 'vault_lock_interval');
",
];

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

pub struct Opened {
    pub database: Database,
    pub moved_aside: Option<PathBuf>,
}

// ! Only corruption justifies moving the file aside; a lock, permission error, or version mismatch must leave data holding secrets untouched.
#[derive(Debug)]
enum OpenFailure {
    Corrupt(String),
    Version(i64),
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
            crate::util::ensure_app_private_dir(parent)
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
        let connection = Self::prepare(connection)
            .map_err(|error| error.into_error("Create the application database"))?;
        Ok(Self::wrap(connection, PathBuf::from(":memory:")))
    }

    pub fn transaction<T>(
        &self,
        context: &'static str,
        work: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> Result<T> {
        let mut connection = self.lock();
        // ! Takes the write lock immediately to avoid SQLite's failed lock upgrade when a read happens before a write.
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

    // ! Rewrites free pages so secrets freed by an earlier write can't be read back from the file; must not run inside a transaction.
    pub fn vacuum(&self, context: &'static str) -> Result<()> {
        let connection = self.lock();
        connection
            .execute_batch("VACUUM")
            .map_err(|error| SteamError::io(context, error_as_io(&error)))
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
        let connection = Self::prepare(connection)?;
        if vacuum_pending(&connection).map_err(OpenFailure::classify)? {
            // ! VACUUM can't run inside a transaction, so it runs here once the upgrade step above has committed; a failure (e.g. low disk space) leaves the flag set for the next start to retry.
            if connection.execute_batch("VACUUM").is_ok() {
                clear_vacuum_pending(&connection).map_err(OpenFailure::classify)?;
            }
        }
        Ok(connection)
    }

    // ! Crossing the `secure_delete` boundary (version 3) sets a persisted flag so the pages it left get rewritten by a VACUUM, retried at the next start if it fails.
    fn prepare(mut connection: Connection) -> std::result::Result<Connection, OpenFailure> {
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(OpenFailure::classify)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(OpenFailure::classify)?;
        // ! From here on, `secure_delete` zeroes freed pages instead of leaving previous plaintext in the file.
        connection
            .pragma_update(None, "secure_delete", true)
            .map_err(OpenFailure::classify)?;
        // ! Runs first so a damaged file is recognised as corrupt rather than reported as a version or query error.
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

    // ! Each step and its version bump commit together, so a crash midway leaves the file at the version it reports.
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
            if target == 3 {
                // ! Set inside the same transaction that crosses the secure_delete boundary, so
                // a crash or a later step's failure can never separate "reached version 3" from
                // "owes a VACUUM for the plaintext secure_delete left behind".
                set_vacuum_pending(&transaction).map_err(OpenFailure::classify)?;
            }
            transaction.commit().map_err(OpenFailure::classify)?;
        }
        Ok(())
    }
}

// ! Tracks whether the secure_delete migration still owes its VACUUM, so a failed VACUUM is retried at next start instead of being silently skipped.
const VACUUM_PENDING_KEY: &str = "_storage_vacuum_pending";

fn vacuum_pending(connection: &Connection) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM setting WHERE key = ?1",
            [VACUUM_PENDING_KEY],
            |_| Ok(()),
        )
        .optional()
        .map(|row| row.is_some())
}

fn set_vacuum_pending(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT OR REPLACE INTO setting (key, value) VALUES (?1, '1')",
        [VACUUM_PENDING_KEY],
    )?;
    Ok(())
}

fn clear_vacuum_pending(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute("DELETE FROM setting WHERE key = ?1", [VACUUM_PENDING_KEY])?;
    Ok(())
}

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

// ! Also moves SQLite's own journal files so the fresh database doesn't inherit a stale write-ahead log.
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

// ! Owner-only: holds secrets, and SQLite's journal files copy the database's permissions.
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
