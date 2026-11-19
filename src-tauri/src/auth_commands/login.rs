use guard_core::authenticator::{
    begin_password_login, begin_qr_login, finish_login, qr_svg, submit_login_code, token_steam_id,
    GuardKind, LoginBegin, LoginResult,
};
use guard_core::error_code::ErrorCode;
use tauri::{AppHandle, State};

use crate::state::{lock_authenticator, AppState};

#[tauri::command]
pub async fn steam_guard_login_password(
    state: State<'_, AppState>,
    account_name: String,
    password: String,
) -> Result<LoginBegin, String> {
    let password = zeroize::Zeroizing::new(password);
    crate::logging::run_async("steam_guard_login_password", async {
        let authenticator = state.authenticator();
        let name = account_name.clone();
        let begin = tauri::async_runtime::spawn_blocking(move || {
            let (session_id, mut login) = lock_authenticator(&authenticator)
                .prepare_login()
                .map_err(|error| error.to_string())?;
            let outcome = begin_password_login(&mut login, &name, &password);
            let mut guard = lock_authenticator(&authenticator);
            match outcome {
                Ok(methods) => {
                    if let Err(error) = guard.store_login(&session_id, login) {
                        guard.cancel_session(&session_id);
                        return Err(error.to_string());
                    }
                    Ok(LoginBegin {
                        session_id,
                        methods,
                        qr_svg: None,
                    })
                }
                Err(error) => {
                    guard.cancel_session(&session_id);
                    Err(crate::logging::auth_error(
                        "steam_guard_login_password",
                        &error,
                    ))
                }
            }
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())??;
        state.remember_login_name(&begin.session_id, &account_name);
        Ok(begin)
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_login_qr(state: State<'_, AppState>) -> Result<LoginBegin, String> {
    crate::logging::run_async("steam_guard_login_qr", async {
        let authenticator = state.authenticator();
        tauri::async_runtime::spawn_blocking(move || {
            let (session_id, mut login) = lock_authenticator(&authenticator)
                .prepare_login()
                .map_err(|error| error.to_string())?;
            let outcome =
                begin_qr_login(&mut login).and_then(|(url, methods)| Ok((qr_svg(&url)?, methods)));
            let mut guard = lock_authenticator(&authenticator);
            match outcome {
                Ok((svg, methods)) => {
                    if let Err(error) = guard.store_login(&session_id, login) {
                        guard.cancel_session(&session_id);
                        return Err(error.to_string());
                    }
                    Ok(LoginBegin {
                        session_id,
                        methods,
                        qr_svg: Some(svg),
                    })
                }
                Err(error) => {
                    guard.cancel_session(&session_id);
                    Err(crate::logging::auth_error("steam_guard_login_qr", &error))
                }
            }
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?
    })
    .await
}

#[tauri::command]
pub async fn steam_guard_login_finish(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    guard_kind: Option<GuardKind>,
    code: Option<String>,
) -> Result<LoginResult, String> {
    let code = code.map(zeroize::Zeroizing::new);
    crate::logging::run_async("steam_guard_login_finish", async {
        let authenticator = state.authenticator();
        let mut login = lock_authenticator(&authenticator)
            .take_login(&session_id)
            .map_err(|error| error.to_string())?;

        let (login, outcome) = tauri::async_runtime::spawn_blocking(move || {
            let result = (|| {
                if let (Some(kind), Some(code)) = (guard_kind, code.as_deref()) {
                    if kind.needs_code() {
                        submit_login_code(&mut login, kind, code)?;
                    }
                }
                finish_login(&mut login)
            })();
            (login, result)
        })
        .await
        .map_err(|_| ErrorCode::WorkerFailed.to_string())?;

        match outcome {
            Ok(tokens) => {
                let steam_id = match token_steam_id(&tokens) {
                    Ok(steam_id) => steam_id.to_string(),
                    Err(error) => {
                        lock_authenticator(&authenticator).cancel_session(&session_id);
                        return Err(error.to_string());
                    }
                };
                let typed_name = state.take_login_name(&session_id);
                let mut guard = lock_authenticator(&authenticator);
                let account_name = typed_name
                    .or_else(|| {
                        guard
                            .accounts()
                            .into_iter()
                            .find(|account| account.steam_id == steam_id)
                            .map(|account| account.account_name)
                    })
                    .unwrap_or_default();
                let result = guard
                    .absorb_login(&session_id, tokens, &account_name)
                    .map_err(|error| error.to_string())?;
                drop(guard);
                state.remember_account_name(&steam_id, &account_name);
                crate::commands::profiles::fetch_in_background(&app, steam_id);
                Ok(result)
            }
            Err(error) => {
                lock_authenticator(&authenticator).put_login(&session_id, login);
                Err(crate::logging::auth_error(
                    "steam_guard_login_finish",
                    &error,
                ))
            }
        }
    })
    .await
}

#[tauri::command(async)]
pub fn steam_guard_login_cancel(state: State<'_, AppState>, session_id: String) {
    crate::logging::write("action=steam_guard_login_cancel status=cancelled");
    state.take_login_name(&session_id);
    let authenticator = state.authenticator();
    lock_authenticator(&authenticator).cancel_session(&session_id);
}
