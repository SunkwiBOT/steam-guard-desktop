// ! Exposes the modules as a library so the `export-bindings` example can reach the DTOs and preference enums without duplicating them.

// ! GTK's prgname is needed before any `AppHandle` exists, so the product name is a literal here.
pub const APP_NAME: &str = "Steam Guard Desktop";
pub mod auth_commands;
pub mod base64;
pub mod clipboard;
pub mod commands;
pub mod locale;
pub mod log_commands;
pub mod logging;

rust_i18n::i18n!("locales", fallback = "en");
pub mod open_url;
pub mod settings;
pub mod state;
pub mod tray;

pub fn show_main_window(app: &tauri::AppHandle) {
    use tauri::Manager;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();

        // ! Wayland refuses focus changes from an unfocused application, so `present()` is used: a tray click would otherwise only raise the window.
        #[cfg(target_os = "linux")]
        {
            if let Ok(gtk_window) = window.gtk_window() {
                use gtk::prelude::GtkWindowExt;
                gtk_window.present();
            }
        }
    }
}
