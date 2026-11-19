// ! `tauri-plugin-opener` runs `xdg-open` without an XDG activation token, so on Wayland a running browser never comes to the front; the token uses `gtk::current_event_time()`, never the page's `MouseEvent.timeStamp`, which does not count on the display server's clock.

use guard_core::error_code::ErrorCode;
use serde::Deserialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export_to = "UrlDestination.ts", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(repr(enum = name))]
pub enum UrlDestination {
    ProjectPage,
}

impl UrlDestination {
    // ! The only URL this command can open: built from the crate's own manifest, never from a page string.
    fn url(self) -> &'static str {
        match self {
            Self::ProjectPage => env!("CARGO_PKG_REPOSITORY"),
        }
    }
}

#[tauri::command(async)]
pub fn open_url(window: tauri::WebviewWindow, destination: UrlDestination) -> Result<(), String> {
    let url = destination.url();
    crate::logging::run("open_url", || dispatch(&window, url))
}

// ! GTK only works on the main thread and an async command runs on a worker, so the launch is handed to the main thread.
#[cfg(target_os = "linux")]
fn dispatch(window: &tauri::WebviewWindow, url: &str) -> Result<(), String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let main_window = window.clone();
    let target = url.to_owned();
    let queued = window.run_on_main_thread(move || {
        let _ = sender.send(open_via_gtk(&main_window, &target));
    });
    let launched = match queued {
        Ok(()) => receiver
            .recv()
            .unwrap_or_else(|_| Err("the main thread dropped the launch".to_owned())),
        Err(error) => Err(error.to_string()),
    };

    match launched {
        Ok(()) => Ok(()),
        Err(reason) => {
            crate::logging::write(&format!(
                "opening a link through GTK failed, falling back: {reason}"
            ));
            open_via_plugin(window, url)
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn dispatch(window: &tauri::WebviewWindow, url: &str) -> Result<(), String> {
    open_via_plugin(window, url)
}

fn open_via_plugin(window: &tauri::WebviewWindow, url: &str) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    window
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|_| ErrorCode::OpenUrlFailed.to_string())
}

#[cfg(target_os = "linux")]
fn open_via_gtk(window: &tauri::WebviewWindow, url: &str) -> Result<(), String> {
    use gtk::prelude::WidgetExt;

    let gtk_window = window.gtk_window().map_err(|error| error.to_string())?;
    let context = gtk_window
        .display()
        .app_launch_context()
        .ok_or_else(|| "the display has no launch context".to_owned())?;
    context.set_timestamp(gtk::current_event_time());
    gio::AppInfo::launch_default_for_uri(url, Some(&context)).map_err(|error| error.to_string())
}
