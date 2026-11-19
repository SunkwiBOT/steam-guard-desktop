use guard_core::error_code::ErrorCode;
use tauri::State;

use super::build_snapshot;
use super::dto::{AppInfoDto, SnapshotDto};
use crate::settings::Settings;
use crate::state::AppState;

fn set_preference(
    name: &'static str,
    state: &State<'_, AppState>,
    change: impl FnOnce(&mut Settings),
    after: impl FnOnce(),
) -> Result<SnapshotDto, String> {
    crate::logging::run(name, || {
        state.update_settings(change)?;
        after();
        Ok(build_snapshot(state))
    })
}

#[tauri::command(async)]
pub fn set_language(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: crate::locale::Language,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_language",
        &state,
        |settings| settings.language = language,
        || crate::tray::refresh_menu(&app),
    )
}

#[tauri::command(async)]
pub fn set_close_to_tray(
    state: State<'_, AppState>,
    close_to_tray: bool,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_close_to_tray",
        &state,
        |settings| settings.close_to_tray = close_to_tray,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_update_interval(
    state: State<'_, AppState>,
    interval: crate::settings::UpdateInterval,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_update_interval",
        &state,
        |settings| settings.update_interval = interval,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_vault_lock_interval(
    state: State<'_, AppState>,
    interval: crate::settings::VaultLockInterval,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_vault_lock_interval",
        &state,
        |settings| settings.vault_lock_interval = interval,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_show_account_names(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    visible: bool,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_show_account_names",
        &state,
        |settings| settings.show_account_names = visible,
        || crate::tray::refresh_menu(&app),
    )
}

#[tauri::command(async)]
pub fn set_steam_ids_visible(
    state: State<'_, AppState>,
    visible: bool,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_steam_ids_visible",
        &state,
        |settings| settings.show_steam_ids = visible,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_blur_codes(state: State<'_, AppState>, enabled: bool) -> Result<SnapshotDto, String> {
    set_preference(
        "set_blur_codes",
        &state,
        |settings| settings.blur_codes = enabled,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_vault_offer_dismissed(state: State<'_, AppState>) -> Result<SnapshotDto, String> {
    set_preference(
        "set_vault_offer_dismissed",
        &state,
        |settings| settings.vault_offer_dismissed = true,
        || {},
    )
}

#[tauri::command(async)]
pub fn set_search_scope(
    state: State<'_, AppState>,
    scope: crate::settings::SearchScope,
) -> Result<SnapshotDto, String> {
    set_preference(
        "set_search_scope",
        &state,
        |settings| settings.search_scope = scope,
        || {},
    )
}

#[tauri::command(async)]
pub fn create_folder(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("create_folder", || {
        state.create_folder(&name, color.as_deref(), icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn update_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("update_folder", || {
        state.update_folder(&id, &name, color.as_deref(), icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn delete_folder(state: State<'_, AppState>, id: String) -> Result<SnapshotDto, String> {
    crate::logging::run("delete_folder", || {
        state.delete_folder(&id)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn assign_folder(
    state: State<'_, AppState>,
    steam_id: String,
    folder_id: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("assign_folder", || {
        state.set_account_folder(&steam_id, folder_id.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_account_color(
    state: State<'_, AppState>,
    steam_id: String,
    color: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_account_color", || {
        state.set_account_color(&steam_id, color.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_folder_color(
    state: State<'_, AppState>,
    id: String,
    color: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_folder_color", || {
        state.set_folder_color(&id, color.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_folder_icon(
    state: State<'_, AppState>,
    id: String,
    icon: Option<String>,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_folder_icon", || {
        state.set_folder_icon(&id, icon.as_deref())?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn app_info(app: tauri::AppHandle) -> AppInfoDto {
    AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        backup_extension: crate::auth_commands::backup::BACKUP_EXTENSION.to_string(),
        folder_icons: crate::settings::FOLDER_ICONS
            .iter()
            .map(|icon| icon.to_string())
            .collect(),
    }
}

#[tauri::command(async)]
pub fn autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    crate::logging::read("autostart_status", || {
        use tauri_plugin_autostart::ManagerExt;
        app.autolaunch()
            .is_enabled()
            .map_err(|_| ErrorCode::AutostartFailed.to_string())
    })
}

#[tauri::command(async)]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    crate::logging::run("set_autostart", || {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|_| ErrorCode::AutostartFailed.to_string())?;
        manager
            .is_enabled()
            .map_err(|_| ErrorCode::AutostartFailed.to_string())
    })
}
