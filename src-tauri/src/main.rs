#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use steam_guard_desktop_lib::{
    auth_commands, clipboard, commands, log_commands, logging, open_url, settings,
    show_main_window, state, tray,
};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

use state::AppState;

fn main() {
    // ! This flag exits before any window, lock file or database is touched, so a self-update check never collides with the running instance.
    if std::env::args().any(|arg| arg == commands::updates::SELFTEST_ARG) {
        std::process::exit(0);
    }
    prepare_linux_display();
    // ! Before the state is built, so a database that cannot be opened leaves a trace.
    logging::init(settings::config_dir());

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(commands::updates::PendingUpdate::new())
        .invoke_handler(tauri::generate_handler![
            commands::snapshots::snapshot,
            commands::snapshots::refresh,
            commands::preferences::app_info,
            commands::preferences::set_language,
            commands::preferences::set_close_to_tray,
            commands::preferences::set_update_interval,
            commands::preferences::set_vault_lock_interval,
            commands::preferences::set_show_account_names,
            commands::preferences::set_steam_ids_visible,
            commands::preferences::set_blur_codes,
            commands::preferences::set_search_scope,
            commands::preferences::set_vault_offer_dismissed,
            commands::preferences::create_folder,
            commands::preferences::update_folder,
            commands::preferences::delete_folder,
            commands::preferences::assign_folder,
            commands::preferences::set_account_color,
            commands::preferences::set_folder_color,
            commands::preferences::set_folder_icon,
            commands::preferences::autostart_status,
            commands::preferences::set_autostart,
            commands::updates::check_for_update,
            commands::updates::download_update,
            commands::updates::apply_update,
            commands::profiles::profile_refresh,
            log_commands::logs_snapshot,
            log_commands::logs_open_directory,
            log_commands::log_frontend_error,
            open_url::open_url,
            clipboard::clipboard_write_text,
            auth_commands::vault::authenticator_unlock,
            auth_commands::vault::authenticator_lock,
            auth_commands::vault::authenticator_set_passphrase,
            auth_commands::guard::steam_guard_code,
            auth_commands::guard::steam_guard_revocation_code,
            auth_commands::guard::account_login_name,
            auth_commands::guard::steam_guard_import,
            auth_commands::guard::steam_guard_confirmations,
            auth_commands::guard::steam_guard_confirmation_action,
            auth_commands::guard::steam_guard_remove,
            auth_commands::guard::steam_guard_forget,
            auth_commands::files::steam_guard_pick_file,
            auth_commands::backup::steam_guard_export_backup,
            auth_commands::backup::steam_guard_import_backup,
            auth_commands::login::steam_guard_login_password,
            auth_commands::login::steam_guard_login_qr,
            auth_commands::login::steam_guard_login_finish,
            auth_commands::login::steam_guard_login_cancel,
            auth_commands::link::steam_guard_link_start,
            auth_commands::link::steam_guard_link_finalize,
            auth_commands::link::steam_guard_link_transfer_start,
            auth_commands::link::steam_guard_link_transfer_finish,
            auth_commands::link::steam_guard_link_revoke,
            auth_commands::link::steam_guard_link_cancel,
            auth_commands::files::steam_guard_clear_picked_file,
            commands::snapshots::default_settings,
            frontend_ready,
        ])
        .setup(|app| {
            let previous_panic_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                // ! Panic payloads can contain secrets: only where it happened is logged.
                let location = info
                    .location()
                    .map(|location| location.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                logging::write(&format!("runtime: panic location={location}"));
                previous_panic_hook(info);
            }));
            logging::write(&format!(
                "start: {} {}",
                app.package_info().name,
                app.package_info().version
            ));
            logging::write(&format!(
                "environment: os={} arch={} debug={}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                cfg!(debug_assertions)
            ));
            {
                let state = app.state::<AppState>();
                let authenticator = state.authenticator();
                let vault = state::lock_authenticator(&authenticator).status();
                logging::write(&format!(
                    "vault: encrypted={} locked={} indexed={} accounts={} load_error={}",
                    vault.encrypted,
                    vault.locked,
                    vault.indexed,
                    vault.count,
                    vault.error.is_some()
                ));
            }
            {
                let state = app.state::<AppState>();
                if let Some(backup) = state.take_corrupt_db_backup() {
                    let locale = state.settings().language.resolved();
                    let message = state::database_recovered_message(locale, &backup);
                    // ! Non-blocking: `setup` runs inside tao's event loop, which owns the GLib idle dispatch a blocking rfd dialog needs.
                    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
                    app.dialog()
                        .message(message)
                        .title(steam_guard_desktop_lib::APP_NAME)
                        .kind(MessageDialogKind::Warning)
                        .buttons(MessageDialogButtons::Ok)
                        .show(|_| {});
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                configure_window(&window);
            }
            let tray_available = tray::install(app.handle()).unwrap_or_else(|error| {
                logging::write("tray: creation_failed");
                eprintln!("Unable to create the tray icon: {error}");
                false
            });
            logging::write(&format!("tray: available={tray_available}"));
            app.state::<AppState>().set_tray_available(tray_available);
            if tray_available {
                tray::refresh_menu(app.handle());
            }
            start_vault_lock_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let close_to_tray = state.settings().close_to_tray;
                if state.tray_available() && close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    state::request_close(app, api);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to build the main window");

    // ! Built, so the single-instance plugin has already exited a second launch and no window
    // exists yet: a database that cannot be opened can show a blocking dialog safely here.
    app.manage(AppState::new());

    app.run(|app_handle, event| match event {
        tauri::RunEvent::Exit => logging::write("exit: normal"),
        // ! Closing every window on macOS leaves the app running in the Dock, and a Dock click raises it again.
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen { .. } => show_main_window(app_handle),
        tauri::RunEvent::ExitRequested { api, .. } if state::exit_blocked(app_handle) => {
            api.prevent_exit();
        }
        _ => {}
    });
}

// ! Without the icon, X11 `WM_CLASS` and Wayland `app_id`, the compositor cannot pair the window with the installed `.desktop` entry.
fn configure_window(window: &tauri::WebviewWindow) {
    if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")) {
        let _ = window.set_icon(icon);
    }

    #[cfg(target_os = "linux")]
    gtk::Window::set_default_icon_name("steamguard");
}

// ! WebKitGTK's DMABUF renderer fails on Wayland with the NVIDIA driver, so it is disabled in that case only.
#[cfg(target_os = "linux")]
fn prepare_linux_display() {
    // ! Before GTK starts: KWin looks up `<app id>.desktop` for the window icon and the bundler names that entry after the product name; the icon name stays `steamguard` because a `steam-` prefix resolves to the Steam client's icon in some themes.
    glib::set_prgname(Some(steam_guard_desktop_lib::APP_NAME));
    glib::set_application_name(steam_guard_desktop_lib::APP_NAME);

    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some() {
        return;
    }

    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|value| value == "wayland");
    let nvidia = std::path::Path::new("/sys/module/nvidia").exists()
        || std::path::Path::new("/proc/driver/nvidia").exists();

    if wayland && nvidia {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

#[cfg(not(target_os = "linux"))]
fn prepare_linux_display() {}

fn start_vault_lock_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(15));
        if app.state::<AppState>().lock_vault_if_due() {
            let _ = app.emit("vault-locked", ());
            tray::refresh_menu(&app);
        }
    });
}

// ! Re-measured every `CLOCK_OFFSET_TTL`: a session running for hours must not keep trusting a stale measurement.
const CLOCK_OFFSET_TTL: Duration = Duration::from_secs(15 * 60);
// ! A measurement that failed (e.g. network not up yet at startup) is retried sooner than a successful one.
const CLOCK_OFFSET_RETRY_TTL: Duration = Duration::from_secs(30);
const CLOCK_DRIFT_THRESHOLD_SECONDS: i64 = 60;

fn start_clock_offset_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        // ! Measured with no lock held: `measure_time_offset` makes the HTTP call itself.
        let result = guard_core::authenticator::measure_time_offset();
        let sleep_for = match result {
            Ok(offset) => {
                let authenticator = app.state::<AppState>().authenticator();
                state::lock_authenticator(&authenticator).set_time_offset(offset);
                logging::write(&format!("clock: steam_offset_seconds={offset}"));
                let _ = app.emit("clock-offset-measured", offset);
                if offset.abs() > CLOCK_DRIFT_THRESHOLD_SECONDS {
                    let _ = app.emit("clock-drift-warning", offset);
                }
                CLOCK_OFFSET_TTL
            }
            Err(error) => {
                logging::write(&format!("clock: steam_offset_unavailable code={error}"));
                CLOCK_OFFSET_RETRY_TTL
            }
        };
        std::thread::sleep(sleep_for);
    });
}

// ! Called by the page once it registered its `clock-offset-measured` and `clock-drift-warning`
// listeners, so the first measurement cannot fire before anything listens.
#[tauri::command(async)]
fn frontend_ready(app: AppHandle) {
    if app.state::<AppState>().mark_frontend_ready() {
        start_clock_offset_loop(app);
    }
}
