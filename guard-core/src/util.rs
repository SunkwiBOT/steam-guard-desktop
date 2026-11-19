use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

pub fn absolute_time(timestamp: Option<i64>) -> String {
    let Some(timestamp) = timestamp else {
        return String::new();
    };

    match time::OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(instant) => {
            let local = instant.to_offset(local_offset_at(timestamp));
            let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]");
            local
                .format(&format)
                .unwrap_or_else(|_| timestamp.to_string())
        }
        Err(_) => timestamp.to_string(),
    }
}

pub fn local_offset_at(timestamp: i64) -> time::UtcOffset {
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .and_then(|instant| time::UtcOffset::local_offset_at(instant).ok())
        .unwrap_or(time::UtcOffset::UTC)
}

// ! No-op on platforms without Unix permission bits.
#[derive(Debug, Clone, Copy)]
pub enum WriteMode {
    // ! Copies whatever is already at the destination, or leaves the
    // platform default when there is nothing there yet: the mode for
    // Steam's own files, which this application must never change.
    PreserveExisting,
    // ! Forces this exact mode regardless of what is already there: the
    // mode for files that hold secrets.
    Private(u32),
}

// ! Rename is atomic but not fsynced; a power loss may leave the previous contents on disk.
pub fn write_atomic(path: &Path, contents: &str, mode: WriteMode) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        match mode {
            WriteMode::Private(_) => create_private_dir_all(parent)?,
            WriteMode::PreserveExisting => fs::create_dir_all(parent)?,
        }
    }

    let temporary = unique_temporary(path);
    #[cfg(unix)]
    let result =
        write_synced(&temporary, path, contents, mode).and_then(|()| fs::rename(&temporary, path));
    #[cfg(not(unix))]
    let result = write_synced(&temporary, contents).and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_synced(
    temporary: &Path,
    #[cfg(unix)] original: &Path,
    contents: &str,
    #[cfg(unix)] mode: WriteMode,
) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if let Some(bits) = mode_bits(original, mode) {
        options.mode(bits);
    }
    let mut file = options.open(temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

// ! Opened with these bits so it is never briefly readable by other users before chmod.
#[cfg(unix)]
fn mode_bits(original: &Path, mode: WriteMode) -> Option<u32> {
    match mode {
        WriteMode::Private(bits) => Some(bits),
        WriteMode::PreserveExisting => fs::metadata(original)
            .ok()
            .map(|metadata| metadata.permissions().mode() & 0o777),
    }
}

fn unique_temporary(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "write".to_string());
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(".{file_name}.{}.{count}.tmp", std::process::id()))
}

// ! Creates missing ancestors at 0700 on Unix only; never touches an existing directory's mode.
pub fn create_private_dir_all(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true).mode(0o700);
        builder.create(path)
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(path)
    }
}

// ! For the app's own data folder, not for an export target: creates it at 0700 and also fixes the mode of one that already exists, so an install upgraded before this check keeps its data private.
pub fn ensure_app_private_dir(path: &Path) -> io::Result<()> {
    create_private_dir_all(path)?;
    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
