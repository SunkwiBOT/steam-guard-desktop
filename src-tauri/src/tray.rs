use std::sync::Mutex;
use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::state::{lock_authenticator, AppState};

pub const TRAY_ID: &str = "steam-guard-desktop-tray";
const ACCOUNT_PREFIX: &str = "account:";
/// Index of the first account entry: after Open and the first separator.
const ACCOUNTS_POSITION: usize = 2;

/// ! Tauri can only replace a tray menu on Windows and macOS; on Linux the
/// existing menu must be edited in place, so its handle is kept.
pub struct TrayMenu(pub Mutex<Option<Menu<tauri::Wry>>>);

/// Reports whether the icon exists: the window must not hide without one.
pub fn install(app: &AppHandle) -> tauri::Result<bool> {
    let menu = build_menu(app)?;
    app.manage(TrayMenu(Mutex::new(Some(menu.clone()))));

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Steam Guard Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(true)
}

fn locale(app: &AppHandle) -> &'static str {
    app.state::<AppState>().settings().language.resolved()
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let locale = locale(app);
    MenuBuilder::new(app)
        .item(&MenuItem::with_id(
            app,
            "open",
            rust_i18n::t!("tray_open", locale = locale),
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "empty",
            rust_i18n::t!("tray_no_accounts", locale = locale),
            false,
            None::<&str>,
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "quit",
            rust_i18n::t!("tray_quit", locale = locale),
            true,
            None::<&str>,
        )?)
        .build()
}

/// ! Queued on the main thread, never awaited: menu calls block until the main
/// thread runs them, so a caller holding a lock (the lock loop, a command
/// holding AppState) would deadlock against a main-thread handler waiting for
/// that same lock.
pub fn refresh_menu(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || update_menu(&handle));
}

fn update_menu(app: &AppHandle) {
    let Some(holder) = app.try_state::<TrayMenu>() else {
        return;
    };
    let Ok(guard) = holder.0.lock() else {
        return;
    };
    let Some(menu) = guard.as_ref() else {
        return;
    };
    let locale = locale(app);
    let Ok(items) = menu.items() else {
        return;
    };

    for (index, item) in items.iter().enumerate().rev() {
        let id = item.id().as_ref().to_string();
        if let tauri::menu::MenuItemKind::MenuItem(item) = item {
            let key = match id.as_str() {
                "open" => Some("tray_open"),
                "quit" => Some("tray_quit"),
                _ => None,
            };
            if let Some(key) = key {
                let _ = item.set_text(rust_i18n::t!(key, locale = locale));
            }
        }
        if id.starts_with(ACCOUNT_PREFIX) || id == "empty" || id == "locked" {
            let _ = menu.remove_at(index);
        }
    }

    let state = app.state::<AppState>();
    let masked = !state.settings().show_account_names;
    let list = crate::commands::snapshots::account_list(&state, masked);

    let mut position = ACCOUNTS_POSITION;
    let mut insert = |id: String, label: String, enabled: bool| {
        if let Ok(item) = MenuItem::with_id(app, id, label, enabled, None::<&str>) {
            if menu.insert(&item, position).is_ok() {
                position += 1;
            }
        }
    };
    if list.accounts.is_empty() {
        insert(
            "empty".to_string(),
            rust_i18n::t!("tray_no_accounts", locale = locale).to_string(),
            false,
        );
        return;
    }
    if list.vault.locked {
        insert(
            "locked".to_string(),
            rust_i18n::t!("tray_locked", locale = locale).to_string(),
            false,
        );
    }
    for account in &list.accounts {
        insert(
            format!("{ACCOUNT_PREFIX}{}", account.steam_id),
            account.display_name.clone(),
            !list.vault.locked,
        );
    }
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "open" => crate::show_main_window(app),
        "quit" => app.exit(0),
        other if other.starts_with(ACCOUNT_PREFIX) => {
            let steam_id = other[ACCOUNT_PREFIX.len()..].to_string();
            copy_code(app, &steam_id);
        }
        _ => {}
    }
}

/// Copies the current code; a vault that locked since the menu was built
/// cannot produce one, so the window is shown instead. The code is never logged.
fn copy_code(app: &AppHandle, steam_id: &str) {
    let code = {
        let state = app.state::<AppState>();
        let authenticator = state.authenticator();
        let guard = lock_authenticator(&authenticator);
        if guard.is_locked() {
            None
        } else {
            Some(guard.code(steam_id).map(|info| info.code))
        }
    };
    match code {
        None => crate::show_main_window(app),
        Some(Err(error)) => crate::logging::write(&format!(
            "action=tray_copy_code status=failed error={}",
            crate::logging::error_category(&error.to_string())
        )),
        Some(Ok(code)) => {
            let handle = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                if crate::clipboard::write_text(&handle, code, "tray_copy_code").is_ok() {
                    crate::logging::write("action=tray_copy_code status=succeeded");
                }
            });
        }
    }
}
