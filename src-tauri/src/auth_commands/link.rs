use guard_core::authenticator::{AuthError, LinkStart, PendingLink, StoredAuthenticator};
use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, State};

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::{lock_authenticator, AppState};

#[tauri::command]
pub async fn steam_guard_link_start(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<LinkStart, String> {
    crate::logging::run_async("steam_guard_link_start", async {
        let numeric: u64 = steam_id
            .parse()
            .map_err(|_| ErrorCode::InvalidSteamId.to_string())?;
        let authenticator = state.authenticator();
        // Shown while the link runs; Steam names the authenticator itself.
        let stored = lock_authenticator(&authenticator).accounts();
        let account_name = stored
            .into_iter()
            .find(|account| account.steam_id == steam_id)
            .map(|account| account.account_name)
            .or_else(|| state.account_name(&steam_id))
            .unwrap_or_else(|| steam_id.clone());
        tauri::async_runtime::spawn_blocking(move || {
            let (session_id, mut pending) = lock_authenticator(&authenticator)
                .prepare_link(numeric, &account_name)
                .map_err(|error| error.to_string())?;
            let outcome = pending.start(&session_id);
            let mut guard = lock_authenticator(&authenticator);
            match outcome {
                Ok(result) => {
                    guard
                        .store_link(&session_id, pending)
                        .map_err(|error| error.to_string())?;
                    Ok(result)
                }
                Err(error) => {
                    guard.finish_link_operation(&session_id);
                    Err(error.to_string())
                }
            }
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

/// Runs one step of a pending link off the main thread, then puts the session
/// back so a failed step can be retried.
async fn link_step<T: Send + 'static>(
    state: &AppState,
    session_id: &str,
    step: impl FnOnce(&mut PendingLink) -> Result<T, AuthError> + Send + 'static,
) -> Result<(PendingLink, Result<T, AuthError>), String> {
    let authenticator = state.authenticator();
    let mut pending = lock_authenticator(&authenticator)
        .take_linker(session_id)
        .map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let result = step(&mut pending);
        (pending, result)
    })
    .await
    .map_err(|_| ErrorCode::WorkerFailed.to_string())
}

fn linked(
    app: &AppHandle,
    state: &AppState,
    stored: StoredAuthenticator,
) -> Result<SnapshotDto, String> {
    state.remember_account_name(&stored.steam_id, &stored.account_name);
    crate::commands::profiles::fetch_in_background(app, stored.steam_id);
    Ok(vault_snapshot(app, state))
}

#[tauri::command]
pub async fn steam_guard_link_finalize(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    confirm_code: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_link_finalize", async {
        let (pending, outcome) = link_step(&state, &session_id, move |pending| {
            pending.finalize(&confirm_code)?;
            pending.verify()?;
            pending
                .take_account()
                .ok_or(AuthError::Code(ErrorCode::AuthenticatorMissing))
        })
        .await?;
        let authenticator = state.authenticator();
        match outcome {
            Ok(account) => {
                let stored = lock_authenticator(&authenticator)
                    .absorb_link(&session_id, account)
                    .map_err(|error| error.to_string())?;
                linked(&app, &state, stored)
            }
            Err(error) => {
                lock_authenticator(&authenticator).put_linker(&session_id, pending);
                Err(error.to_string())
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_transfer_start(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    crate::logging::run_async("steam_guard_link_transfer_start", async {
        let (pending, outcome) =
            link_step(&state, &session_id, |pending| pending.transfer_start()).await?;
        let authenticator = state.authenticator();
        lock_authenticator(&authenticator).put_linker(&session_id, pending);
        outcome.map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_transfer_finish(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    sms_code: String,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_link_transfer_finish", async {
        let (pending, outcome) = link_step(&state, &session_id, move |pending| {
            pending.transfer_finish(&sms_code)
        })
        .await?;
        let authenticator = state.authenticator();
        match outcome {
            Ok(account) => {
                let stored = lock_authenticator(&authenticator)
                    .absorb_link(&session_id, account)
                    .map_err(|error| error.to_string())?;
                linked(&app, &state, stored)
            }
            Err(error) => {
                lock_authenticator(&authenticator).put_linker(&session_id, pending);
                Err(error.to_string())
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_link_revoke(
    state: State<'_, AppState>,
    session_id: String,
    revocation_code: String,
) -> Result<(), String> {
    let revocation_code = zeroize::Zeroizing::new(revocation_code);
    crate::logging::run_async("steam_guard_link_revoke", async {
        let (pending, outcome) = link_step(&state, &session_id, move |pending| {
            pending.revoke(&revocation_code)
        })
        .await?;
        let authenticator = state.authenticator();
        let mut guard = lock_authenticator(&authenticator);
        if outcome.is_ok() {
            guard.finish_link_operation(&session_id);
            guard.cancel_session(&session_id);
        } else {
            guard.put_linker(&session_id, pending);
        }
        outcome.map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command(async)]
pub fn steam_guard_link_cancel(state: State<'_, AppState>, session_id: String) {
    crate::logging::write("action=steam_guard_link_cancel status=cancelled");
    let authenticator = state.authenticator();
    lock_authenticator(&authenticator).cancel_session(&session_id);
}
