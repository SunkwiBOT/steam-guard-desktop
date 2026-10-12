//! ! File paths handed to the import commands come from a native open dialog
//! opened here, so the page never names a file on disk itself.

use guard_core::error_code::ErrorCode;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use super::backup::BACKUP_EXTENSION;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PickKind {
    Mafile,
    Backup,
}

/// `None` means cancelled.
#[tauri::command]
pub async fn steam_guard_pick_file(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: PickKind,
) -> Result<Option<String>, String> {
    crate::logging::run_async("steam_guard_pick_file", async {
        let locale = state.settings().language.resolved();
        tauri::async_runtime::spawn_blocking(move || {
            let (title, filter, extension) = match kind {
                PickKind::Mafile => ("pick_mafile_title", "mafile_file_filter", "maFile"),
                PickKind::Backup => ("pick_backup_title", "backup_file_filter", BACKUP_EXTENSION),
            };
            let mut dialog = app
                .dialog()
                .file()
                .set_title(rust_i18n::t!(title, locale = locale))
                .add_filter(rust_i18n::t!(filter, locale = locale), &[extension]);
            if let Some(window) = app.get_webview_window("main") {
                dialog = dialog.set_parent(&window);
            }
            let Some(chosen) = dialog.blocking_pick_file() else {
                return Ok(None);
            };
            chosen
                .into_path()
                .map(|path| Some(path.to_string_lossy().into_owned()))
                .map_err(|_| ErrorCode::IoError.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}
