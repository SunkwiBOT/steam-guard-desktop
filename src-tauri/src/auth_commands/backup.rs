use std::path::Path;

use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, State};

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::{lock_authenticator, AppState};

#[derive(serde::Serialize)]
pub struct BackupImportDto {
    pub imported: usize,
    pub skipped: usize,
    pub snapshot: SnapshotDto,
}

/// Extension of the backup files, shared with the switcher so either app can
/// import the other's export.
pub(super) const BACKUP_EXTENSION: &str = "steamguard";

/// ! The destination comes from a native save dialog opened here, never from
/// the page, so a script in the webview can never choose where the secrets are
/// written. `None` means cancelled.
#[tauri::command]
pub async fn steam_guard_export_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    steam_id: Option<String>,
    password: Option<String>,
) -> Result<Option<usize>, String> {
    use tauri::Manager;
    use tauri_plugin_dialog::DialogExt;

    let password = password.map(zeroize::Zeroizing::new);
    crate::logging::run_async("steam_guard_export_backup", async {
        let locale = state.settings().language.resolved();
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut dialog = app
                .dialog()
                .file()
                .set_title(rust_i18n::t!("backup_export_title", locale = locale))
                .set_file_name(format!("authenticators.{BACKUP_EXTENSION}"))
                .add_filter(
                    rust_i18n::t!("backup_file_filter", locale = locale),
                    &[BACKUP_EXTENSION],
                );
            if let Some(window) = app.get_webview_window("main") {
                dialog = dialog.set_parent(&window);
            }
            let Some(chosen) = dialog.blocking_save_file() else {
                return Ok(None);
            };
            let path = chosen
                .into_path()
                .map_err(|_| ErrorCode::BackupTargetUnsafe.to_string())?;
            lock_authenticator(&authenticator)
                .export_backup(
                    &path,
                    steam_id.as_deref(),
                    password.as_deref().map(|value| value.as_str()),
                )
                .map(Some)
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_import_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    password: Option<String>,
) -> Result<BackupImportDto, String> {
    let password = password.map(zeroize::Zeroizing::new);
    crate::logging::run_async("steam_guard_import_backup", async {
        let authenticator = state.authenticator();
        let result = tauri::async_runtime::spawn_blocking(move || {
            lock_authenticator(&authenticator)
                .import_backup(
                    Path::new(&path),
                    password.as_deref().map(|value| value.as_str()),
                )
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        if result.imported > 0 {
            crate::commands::profiles::fetch_missing_in_background(&app);
        }
        Ok(BackupImportDto {
            imported: result.imported,
            skipped: result.skipped,
            snapshot: vault_snapshot(&app, &state),
        })
    })
    .await
}
