use std::path::PathBuf;

mod model;
mod preferences;
mod store;

pub use model::{Folder, Settings};
pub use preferences::{SearchScope, UpdateInterval, VaultLockInterval};
pub use store::SettingsStore;

/// `%APPDATA%\steam-guard-desktop` on Windows, `~/Library/Application Support/steam-guard-desktop`
/// on macOS, `$XDG_CONFIG_HOME/steam-guard-desktop` (or `~/.config`) elsewhere.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-guard-desktop")
}

pub fn database_path() -> PathBuf {
    config_dir().join("steam-guard-desktop.db")
}
