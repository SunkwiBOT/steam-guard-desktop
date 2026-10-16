use std::path::Path;

use guard_core::authenticator::{
    confirmation_action, confirmations, remove_authenticator, AuthError, CodeInfo, ConfirmationItem,
};
use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, State};

use super::vault::vault_snapshot;
use crate::commands::SnapshotDto;
use crate::state::{lock_authenticator, AppState};

#[tauri::command(async)]
pub fn steam_guard_code(state: State<'_, AppState>, steam_id: String) -> Result<CodeInfo, String> {
    crate::logging::read("steam_guard_code", || {
        let authenticator = state.authenticator();
        let guard = lock_authenticator(&authenticator);
        guard.code(&steam_id).map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn steam_guard_revocation_code(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<String, String> {
    crate::logging::read("steam_guard_revocation_code", || {
        let authenticator = state.authenticator();
        let guard = lock_authenticator(&authenticator);
        guard
            .revocation_code(&steam_id)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn account_login_name(state: State<'_, AppState>, steam_id: String) -> Result<String, String> {
    crate::logging::read("account_login_name", || {
        let stored = {
            let authenticator = state.authenticator();
            let guard = lock_authenticator(&authenticator);
            guard.account_name(&steam_id)
        };
        // A pending link may only know the name typed at sign-in.
        match stored {
            Ok(name) if !name.is_empty() => Ok(name),
            Ok(_) | Err(AuthError::Missing) => state
                .account_name(&steam_id)
                .ok_or_else(|| ErrorCode::AuthenticatorMissing.to_string()),
            Err(error) => Err(error.to_string()),
        }
    })
}

/// The Steam ID comes from the file itself.
#[tauri::command]
pub async fn steam_guard_import(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    passkey: Option<String>,
) -> Result<SnapshotDto, String> {
    let passkey = passkey.map(zeroize::Zeroizing::new);
    crate::logging::run_async("steam_guard_import", async {
        let authenticator = state.authenticator();
        let stored = tauri::async_runtime::spawn_blocking(move || {
            let mut guard = lock_authenticator(&authenticator);
            guard
                .import(
                    Path::new(&path),
                    passkey.as_deref().map(|value| value.as_str()),
                    None,
                )
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        crate::commands::profiles::fetch_in_background(&app, stored.steam_id);
        Ok(vault_snapshot(&app, &state))
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_confirmations(
    state: State<'_, AppState>,
    steam_id: String,
) -> Result<Vec<ConfirmationItem>, String> {
    crate::logging::run_async("steam_guard_confirmations", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut account = lock_authenticator(&authenticator)
                .confirmation_account(&steam_id)
                .map_err(|error| error.to_string())?;
            let result = confirmations(&mut account).map_err(|error| error.to_string())?;
            lock_authenticator(&authenticator)
                .remember_tokens(&account)
                .map_err(|error| error.to_string())?;
            Ok(result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_confirmation_action(
    state: State<'_, AppState>,
    steam_id: String,
    id: String,
    nonce: String,
    accept: bool,
) -> Result<(), String> {
    crate::logging::run_async("steam_guard_confirmation_action", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let mut account = lock_authenticator(&authenticator)
                .confirmation_account(&steam_id)
                .map_err(|error| error.to_string())?;
            confirmation_action(&mut account, &id, &nonce, accept)
                .map_err(|error| error.to_string())?;
            // An accepted action must not be reported as failed just because
            // saving refreshed tokens failed afterwards.
            if let Err(error) = lock_authenticator(&authenticator).remember_tokens(&account) {
                crate::logging::write(&format!(
                    "action=steam_guard_confirmation_action stage=save_session status=failed error={}",
                    crate::logging::error_category(&error.to_string())
                ));
            }
            Ok::<(), String>(())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

/// Revokes the authenticator on Steam, then forgets it locally.
#[tauri::command]
pub async fn steam_guard_remove(
    app: AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
    revocation_code: Option<String>,
) -> Result<SnapshotDto, String> {
    let revocation_code = revocation_code.map(zeroize::Zeroizing::new);
    crate::logging::run_async("steam_guard_remove", async {
        let authenticator = state.authenticator();
        let id = steam_id.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let mut account = lock_authenticator(&authenticator)
                .confirmation_account(&id)
                .map_err(|error| error.to_string())?;
            remove_authenticator(&mut account, revocation_code.map(|code| code.to_string()))
                .map_err(|error| error.to_string())?;
            lock_authenticator(&authenticator)
                .forget_revoked(&id)
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        forget_profile(&state, &steam_id);
        state.forget_account(&steam_id);
        Ok(vault_snapshot(&app, &state))
    })
    .await
}

/// Local only: the authenticator stays active on Steam, so the interface must
/// have shown the warning first.
#[tauri::command]
pub async fn steam_guard_forget(
    app: AppHandle,
    state: State<'_, AppState>,
    steam_id: String,
    confirmed: bool,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("steam_guard_forget", async {
        if !confirmed {
            return Err(ErrorCode::ConfirmationRequired.to_string());
        }
        let authenticator = state.authenticator();
        let id = steam_id.clone();
        tauri::async_runtime::spawn_blocking(move || {
            lock_authenticator(&authenticator)
                .forget_local(&id)
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        forget_profile(&state, &steam_id);
        state.forget_account(&steam_id);
        Ok(vault_snapshot(&app, &state))
    })
    .await
}

/// The cached profile is only presentation: a failure to drop it is logged, never reported.
fn forget_profile(state: &AppState, steam_id: &str) {
    let removed = steam_id
        .parse::<u64>()
        .map_err(|_| ErrorCode::InvalidSteamId.to_string())
        .and_then(|numeric| {
            state
                .profiles
                .remove(numeric)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = removed {
        crate::logging::write(&format!(
            "action=forget_profile status=failed error={}",
            crate::logging::error_category(&error)
        ));
    }
}
