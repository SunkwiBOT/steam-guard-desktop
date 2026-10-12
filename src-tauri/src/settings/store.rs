use std::collections::BTreeMap;

use guard_core::error_code::ErrorCode;
use guard_core::storage::Database;
use rusqlite::{Connection, Transaction};
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Folder, Settings};

pub struct SettingsStore {
    database: Database,
    data: Settings,
}

impl SettingsStore {
    /// Reads the settings from the database. A database that cannot be read is kept
    /// aside by `Database::open`, and the defaults are used for this run.
    pub fn load(database: Database) -> Self {
        let data = database
            .read("Read settings", read_settings)
            .unwrap_or_default();
        Self { database, data }
    }

    pub fn data(&self) -> &Settings {
        &self.data
    }

    /// Applies a change and writes it, putting the previous values back in
    /// memory when the write fails so the interface never shows a change that
    /// is not on disk.
    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) -> Result<(), String> {
        let previous = self.data.clone();
        change(&mut self.data);
        let saved = self.save();
        if saved.is_err() {
            self.data = previous;
        }
        saved
    }

    fn save(&self) -> Result<(), String> {
        self.database
            .transaction("Save settings", |transaction| {
                write_settings(transaction, &self.data)
            })
            .map_err(|_| ErrorCode::SettingsSaveFailed.to_string())
    }
}

fn read_value<T: DeserializeOwned>(connection: &Connection, key: &str) -> Option<T> {
    let text: String = connection
        .query_row("SELECT value FROM setting WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .ok()?;
    serde_json::from_str(&text).ok()
}

fn read_settings(connection: &Connection) -> rusqlite::Result<Settings> {
    let defaults = Settings::default();
    let folders = connection
        .prepare("SELECT id, name, color, icon FROM folder ORDER BY position")?
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let pairs = |sql: &str| -> rusqlite::Result<BTreeMap<String, String>> {
        connection
            .prepare(sql)?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect()
    };
    Ok(Settings {
        language: read_value(connection, "language").unwrap_or(defaults.language),
        close_to_tray: read_value(connection, "close_to_tray").unwrap_or(defaults.close_to_tray),
        show_account_names: read_value(connection, "show_account_names")
            .unwrap_or(defaults.show_account_names),
        blur_codes: read_value(connection, "blur_codes").unwrap_or(defaults.blur_codes),
        update_interval: read_value(connection, "update_interval")
            .unwrap_or(defaults.update_interval),
        search_scope: read_value(connection, "search_scope").unwrap_or(defaults.search_scope),
        vault_lock_interval: read_value(connection, "vault_lock_interval")
            .unwrap_or(defaults.vault_lock_interval),
        last_update_check_at: read_value::<Option<u64>>(connection, "last_update_check_at")
            .flatten(),
        folders,
        account_folders: pairs("SELECT steam_id, folder_id FROM account_folder")?,
        account_colors: pairs("SELECT steam_id, color FROM account_color")?,
    })
}

fn write_settings(transaction: &Transaction<'_>, settings: &Settings) -> rusqlite::Result<()> {
    transaction.execute_batch(
        "DELETE FROM setting;
         DELETE FROM folder;
         DELETE FROM account_folder;
         DELETE FROM account_color;",
    )?;
    put_setting(transaction, "language", &settings.language)?;
    put_setting(transaction, "close_to_tray", &settings.close_to_tray)?;
    put_setting(
        transaction,
        "show_account_names",
        &settings.show_account_names,
    )?;
    put_setting(transaction, "blur_codes", &settings.blur_codes)?;
    put_setting(transaction, "update_interval", &settings.update_interval)?;
    put_setting(transaction, "search_scope", &settings.search_scope)?;
    put_setting(
        transaction,
        "vault_lock_interval",
        &settings.vault_lock_interval,
    )?;
    put_setting(
        transaction,
        "last_update_check_at",
        &settings.last_update_check_at,
    )?;
    for (position, folder) in settings.folders.iter().enumerate() {
        transaction.execute(
            "INSERT INTO folder (id, position, name, color, icon) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &folder.id,
                position as i64,
                &folder.name,
                &folder.color,
                &folder.icon,
            ),
        )?;
    }
    for (steam_id, folder_id) in &settings.account_folders {
        transaction.execute(
            "INSERT INTO account_folder (steam_id, folder_id) VALUES (?1, ?2)",
            (steam_id, folder_id),
        )?;
    }
    for (steam_id, color) in &settings.account_colors {
        transaction.execute(
            "INSERT INTO account_color (steam_id, color) VALUES (?1, ?2)",
            (steam_id, color),
        )?;
    }
    Ok(())
}

/// Each preference is stored as its JSON text, so the enums keep their serde names.
fn put_setting<T: Serialize>(
    transaction: &Transaction<'_>,
    key: &str,
    value: &T,
) -> rusqlite::Result<()> {
    let text = serde_json::to_string(value).unwrap_or_default();
    transaction.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)",
        (key, text),
    )?;
    Ok(())
}
