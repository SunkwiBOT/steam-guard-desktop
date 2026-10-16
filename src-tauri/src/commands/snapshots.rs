use std::collections::BTreeMap;

use guard_core::authenticator::VaultStatus;
use guard_core::profile::Profile;
use tauri::State;

use super::dto::{AccountDto, FolderDto, SettingsDto, SnapshotDto, VaultStatusDto};
use crate::base64;
use crate::settings::Settings;
use crate::state::{lock_authenticator, AppState};

pub(crate) struct AccountList {
    pub accounts: Vec<AccountDto>,
    pub vault: VaultStatus,
}

/// Lists the accounts the way the cards and the tray show them. While the
/// vault is locked only the public index and the cached profiles are known.
pub(crate) fn account_list(state: &AppState, masked: bool) -> AccountList {
    let settings = state.settings();
    let (vault, stored, configured, pending) = {
        let authenticator = state.authenticator();
        let guard = lock_authenticator(&authenticator);
        (
            guard.status(),
            guard.accounts(),
            guard.configured_ids(),
            guard.pending_ids(),
        )
    };
    let profiles: BTreeMap<String, Profile> = match state.profiles.all() {
        Ok(profiles) => profiles
            .into_iter()
            .map(|profile| (profile.steam_id.to_string(), profile))
            .collect(),
        Err(error) => {
            crate::logging::write(&format!(
                "action=read_profiles status=failed error={}",
                crate::logging::error_category(&error.to_string())
            ));
            BTreeMap::new()
        }
    };

    let dto = |steam_id: String, account_name: String, has_tokens: bool, pending: bool| {
        account_dto(
            steam_id,
            account_name,
            has_tokens,
            pending,
            &profiles,
            &settings,
            masked,
        )
    };
    let mut accounts: Vec<AccountDto> = if vault.locked {
        configured
            .into_iter()
            .map(|steam_id| dto(steam_id, String::new(), false, false))
            .collect()
    } else {
        stored
            .into_iter()
            .map(|account| {
                dto(
                    account.steam_id,
                    account.account_name,
                    account.has_tokens,
                    false,
                )
            })
            .chain(pending.into_iter().map(|steam_id| {
                let account_name = state.account_name(&steam_id).unwrap_or_default();
                dto(steam_id, account_name, false, true)
            }))
            .collect()
    };
    accounts.sort_by_cached_key(|account| {
        (
            account.display_name.to_lowercase(),
            account.steam_id.clone(),
        )
    });
    AccountList { accounts, vault }
}

fn account_dto(
    steam_id: String,
    account_name: String,
    has_tokens: bool,
    pending: bool,
    profiles: &BTreeMap<String, Profile>,
    settings: &Settings,
    masked: bool,
) -> AccountDto {
    let profile = profiles.get(&steam_id);
    let persona_name = profile
        .map(|profile| profile.persona_name.clone())
        .unwrap_or_default();
    let account_name = if masked { String::new() } else { account_name };
    let display_name = [persona_name.as_str(), account_name.as_str()]
        .into_iter()
        .find(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback_name(&steam_id, settings.show_steam_ids));
    AccountDto {
        initial: initial(&display_name),
        avatar: profile
            .and_then(|profile| profile.avatar.as_ref())
            .and_then(|(bytes, mime)| base64::data_url(bytes, mime)),
        profile_fetched_at: profile.map(|profile| profile.fetched_at),
        color: settings.account_color(&steam_id).map(str::to_string),
        folder_id: settings.account_folder(&steam_id).map(str::to_string),
        steam_id,
        account_name,
        persona_name,
        display_name,
        has_tokens,
        pending,
    }
}

/// ! Hidden Steam IDs are masked completely, like the dialog does.
fn fallback_name(steam_id: &str, show_steam_ids: bool) -> String {
    if show_steam_ids {
        steam_id.to_string()
    } else {
        "******".to_string()
    }
}

fn initial(display_name: &str) -> String {
    match display_name.trim().chars().next() {
        Some(character) => character.to_uppercase().to_string(),
        None => "?".to_string(),
    }
}

pub(crate) fn build_snapshot(state: &AppState) -> SnapshotDto {
    let (settings, revision, lock_in_seconds) = {
        let inner = state.lock();
        let settings = inner.settings.data().clone();
        let lock_in_seconds = inner.unlocked_at.and_then(|unlocked_at| {
            settings
                .vault_lock_interval
                .seconds()
                .map(|seconds| seconds.saturating_sub(unlocked_at.elapsed().as_secs()))
        });
        (settings, state.next_snapshot_revision(), lock_in_seconds)
    };
    let masked = !settings.show_account_names;
    let list = account_list(state, masked);
    let folders = settings
        .folders
        .iter()
        .map(|folder| FolderDto {
            id: folder.id.clone(),
            name: folder.name.clone(),
            account_count: list
                .accounts
                .iter()
                .filter(|account| account.folder_id.as_deref() == Some(folder.id.as_str()))
                .count(),
            system: false,
            color: folder.color.clone(),
            icon: folder.icon.clone(),
        })
        .collect();
    SnapshotDto {
        revision,
        accounts: list.accounts,
        vault: VaultStatusDto {
            status: list.vault,
            lock_in_seconds,
        },
        settings: SettingsDto {
            language: settings.language,
            resolved_language: settings.language.resolved().to_string(),
            close_to_tray: settings.close_to_tray,
            show_account_names: settings.show_account_names,
            show_steam_ids: settings.show_steam_ids,
            blur_codes: settings.blur_codes,
            update_interval: settings.update_interval,
            search_scope: settings.search_scope,
            vault_lock_interval: settings.vault_lock_interval,
            last_update_check_at: settings.last_update_check_at,
        },
        folders,
        masked,
    }
}

#[tauri::command(async)]
pub fn snapshot(state: State<'_, AppState>) -> SnapshotDto {
    build_snapshot(&state)
}

#[tauri::command(async)]
pub fn refresh(app: tauri::AppHandle, state: State<'_, AppState>) -> SnapshotDto {
    crate::logging::write("action=refresh status=requested");
    crate::tray::refresh_menu(&app);
    build_snapshot(&state)
}
