use guard_core::error_code::ErrorCode;
use tauri::State;

use super::build_snapshot;
use super::dto::{AppInfoDto, SnapshotDto};
use crate::state::AppState;

const REPOSITORY_URL: &str = "https://github.com/SunkwiBOT/steam-guard-desktop";

#[tauri::command(async)]
pub fn set_language(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    language: crate::locale::Language,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_language", || {
        state.update_settings(|settings| settings.language = language)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_close_to_tray(
    state: State<'_, AppState>,
    close_to_tray: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_close_to_tray", || {
        state.update_settings(|settings| settings.close_to_tray = close_to_tray)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_update_interval(
    state: State<'_, AppState>,
    interval: crate::settings::UpdateInterval,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_update_interval", || {
        state.update_settings(|settings| settings.update_interval = interval)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_vault_lock_interval(
    state: State<'_, AppState>,
    interval: crate::settings::VaultLockInterval,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_vault_lock_interval", || {
        state.update_settings(|settings| settings.vault_lock_interval = interval)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_show_account_names(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    visible: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_show_account_names", || {
        state.update_settings(|settings| settings.show_account_names = visible)?;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_steam_ids_visible(
    state: State<'_, AppState>,
    visible: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_steam_ids_visible", || {
        state.update_settings(|settings| settings.show_steam_ids = visible)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_blur_codes(state: State<'_, AppState>, enabled: bool) -> Result<SnapshotDto, String> {
    crate::logging::run("set_blur_codes", || {
        state.update_settings(|settings| settings.blur_codes = enabled)?;
        Ok(build_snapshot(&state))
    })
}

#[tauri::command(async)]
pub fn set_search_scope(
    state: State<'_, AppState>,
    scope: crate::settings::SearchScope,
) -> Result<SnapshotDto, String> {
    crate::logging::run("set_search_scope", || {
        state.update_settings(|settings| settings.search_scope = scope)?;
        Ok(build_snapshot(&state))
    })
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
pub fn set_window_active(state: State<'_, AppState>, active: bool) {
    state.set_window_active(active);
}

#[tauri::command(async)]
pub fn app_info(app: tauri::AppHandle) -> AppInfoDto {
    AppInfoDto {
        name: app.package_info().name.clone(),
        version: app.package_info().version.to_string(),
        repository: REPOSITORY_URL.to_string(),
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
