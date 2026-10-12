//! Public community profiles (persona name and avatar) cached next to the
//! vault so the cards can show them while it is locked.

use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, Emitter, Manager, State};

use super::build_snapshot;
use super::dto::SnapshotDto;
use crate::state::{lock_authenticator, AppState};

fn fetch_and_store(state: &AppState, steam_id: &str) -> Result<(), String> {
    let numeric: u64 = steam_id
        .parse()
        .map_err(|_| ErrorCode::InvalidSteamId.to_string())?;
    let profile = guard_core::profile::fetch(numeric).map_err(|error| error.to_string())?;
    state
        .profiles
        .put(&profile)
        .map_err(|error| error.to_string())
}

/// Fetches one profile off the main thread and tells the interface and the
/// tray when it is stored; a failure is only logged as a category.
pub(crate) fn fetch_in_background(app: &AppHandle, steam_id: String) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        match fetch_and_store(&state, &steam_id) {
            Ok(()) => {
                crate::logging::write("action=profile_fetch status=succeeded");
                crate::tray::refresh_menu(&app);
                let _ = app.emit("accounts-changed", ());
            }
            Err(error) => crate::logging::write(&format!(
                "action=profile_fetch status=failed error={}",
                crate::logging::error_category(&error)
            )),
        }
    });
}

/// After a backup import the imported ids are not reported: every stored
/// account without a cached profile is fetched.
pub(crate) fn fetch_missing_in_background(app: &AppHandle) {
    let state = app.state::<AppState>();
    let ids: Vec<String> = {
        let authenticator = state.authenticator();
        let guard = lock_authenticator(&authenticator);
        guard.configured_ids()
    };
    for steam_id in ids {
        let cached = steam_id
            .parse::<u64>()
            .ok()
            .and_then(|numeric| state.profiles.get(numeric).ok().flatten());
        if cached.is_none() {
            fetch_in_background(app, steam_id);
        }
    }
}

#[tauri::command]
pub async fn profile_refresh(
    app: AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("profile_refresh", async {
        let worker = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            fetch_and_store(&worker.state::<AppState>(), &steam_id)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        crate::tray::refresh_menu(&app);
        Ok(build_snapshot(&state))
    })
    .await
}
