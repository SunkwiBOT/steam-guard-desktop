use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> f64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(_) => 0.0,
    }
}

/// ! Writes a synced temporary file, then renames it over `path`: readers see
/// ! the old or the new contents, never a partial file. The rename is not
/// ! fsynced, so after a power loss the previous contents may still be on disk.
pub fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temporary = unique_temporary(path);
    let result = write_synced(&temporary, contents).and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_synced(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

fn unique_temporary(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "steam-guard-desktop".to_string());
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(".{file_name}.{}.{count}.tmp", std::process::id()))
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

/// Local UTC offset in force at `timestamp`, so instants on both sides of a
/// daylight-saving change land on their own day and hour. Falls back to UTC
/// when the platform cannot tell.
pub fn local_offset_at(timestamp: i64) -> time::UtcOffset {
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .and_then(|instant| time::UtcOffset::local_offset_at(instant).ok())
        .unwrap_or(time::UtcOffset::UTC)
}
