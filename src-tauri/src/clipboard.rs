//! Native clipboard writes, independent of webview focus and user activation.

use guard_core::error_code::ErrorCode;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn clipboard_write_text(app: AppHandle, text: String) -> Result<(), String> {
    crate::logging::run_async("clipboard_write_text", async {
        tauri::async_runtime::spawn_blocking(move || write_text(&app, text, "clipboard_write_text"))
            .await
            .map_err(|_| {
                crate::logging::write("action=clipboard_write_text error=worker_failed");
                ErrorCode::ClipboardUnavailable.as_str().to_string()
            })?
    })
    .await
}

/// Never logs the copied text: it may be a recovery or login code.
pub fn write_text(app: &AppHandle, text: String, action: &str) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|error| {
        crate::logging::write(&format!(
            "action={action} stage=native_write error={}",
            crate::logging::clipboard_error_category(&error.to_string())
        ));
        ErrorCode::ClipboardUnavailable.as_str().to_string()
    })
}
