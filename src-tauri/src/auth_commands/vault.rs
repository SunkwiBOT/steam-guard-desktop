use guard_core::authenticator::{AuthError, Authenticator, StoredAuthenticator, VaultStatus};
use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, State};

use crate::commands::{build_snapshot, SnapshotDto};
use crate::state::{lock_authenticator, AppState};

/// Snapshot after a change to the vault or its accounts: logs the new vault
/// state and rebuilds the tray, which shows the same list.
pub(super) fn vault_snapshot(app: &AppHandle, state: &AppState) -> SnapshotDto {
    let snapshot = build_snapshot(state);
    let vault = &snapshot.vault;
    crate::logging::write(&format!(
        "vault: encrypted={} locked={} indexed={} accounts={}",
        vault.status.encrypted, vault.status.locked, vault.status.indexed, vault.status.count
    ));
    crate::tray::refresh_menu(app);
    snapshot
}

#[tauri::command(async)]
pub fn authenticator_status(state: State<'_, AppState>) -> VaultStatus {
    let authenticator = state.authenticator();
    let guard = lock_authenticator(&authenticator);
    guard.status()
}

#[tauri::command(async)]
pub fn authenticator_accounts(state: State<'_, AppState>) -> Vec<StoredAuthenticator> {
    let authenticator = state.authenticator();
    let guard = lock_authenticator(&authenticator);
    guard.accounts()
}

async fn update_vault(
    app: &AppHandle,
    state: &AppState,
    action: impl FnOnce(&mut Authenticator) -> Result<(), AuthError> + Send + 'static,
) -> Result<SnapshotDto, String> {
    let authenticator = state.authenticator();
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_authenticator(&authenticator);
        action(&mut guard).map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
    // ! vault_snapshot locks the authenticator again: keep it outside the blocking closure.
    let snapshot = vault_snapshot(app, state);
    if snapshot.vault.status.encrypted && !snapshot.vault.status.locked {
        state.note_vault_unlocked();
    } else {
        state.note_vault_locked();
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn authenticator_unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    passphrase: String,
) -> Result<SnapshotDto, String> {
    let passphrase = zeroize::Zeroizing::new(passphrase);
    crate::logging::run_async("authenticator_unlock", async {
        update_vault(&app, &state, move |guard| {
            guard.unlock(&passphrase).map(|_| ())
        })
        .await
    })
    .await
}

#[tauri::command]
pub async fn authenticator_lock(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SnapshotDto, String> {
    crate::logging::run_async("authenticator_lock", async {
        update_vault(&app, &state, |guard| guard.lock()).await
    })
    .await
}

#[tauri::command]
pub async fn authenticator_set_passphrase(
    app: AppHandle,
    state: State<'_, AppState>,
    passphrase: Option<String>,
    current_password: Option<String>,
) -> Result<SnapshotDto, String> {
    let passphrase = passphrase.map(zeroize::Zeroizing::new);
    let current_password = current_password.map(zeroize::Zeroizing::new);
    crate::logging::run_async("authenticator_set_passphrase", async {
        update_vault(&app, &state, move |guard| {
            guard
                .set_passphrase(
                    passphrase
                        .as_deref()
                        .map(|value| value.as_str().to_string()),
                    current_password
                        .as_deref()
                        .map(|value| value.as_str().to_string()),
                )
                .map(|_| ())
        })
        .await
    })
    .await
}
