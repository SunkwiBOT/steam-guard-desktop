use std::path::PathBuf;

mod model;
mod preferences;
mod store;

pub use model::{Folder, Settings, FOLDER_ICONS};
pub use preferences::{SearchScope, UpdateInterval, VaultLockInterval};
pub use store::SettingsStore;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-guard-desktop")
}

pub fn database_path() -> PathBuf {
    config_dir().join("steam-guard-desktop.db")
}
