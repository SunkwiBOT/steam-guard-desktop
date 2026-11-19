use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export_to = "SettingsDto.ts")]
pub struct SettingsDto {
    pub language: crate::locale::Language,
    pub resolved_language: String,
    pub close_to_tray: bool,
    pub show_account_names: bool,
    pub show_steam_ids: bool,
    pub blur_codes: bool,
    pub update_interval: crate::settings::UpdateInterval,
    pub search_scope: crate::settings::SearchScope,
    pub vault_lock_interval: crate::settings::VaultLockInterval,
    #[ts(type = "number | null")]
    pub last_update_check_at: Option<u64>,
    pub vault_offer_dismissed: bool,
}

#[derive(Clone, Serialize, TS)]
#[ts(export_to = "AccountDto.ts")]
pub struct AccountDto {
    pub steam_id: String,
    pub account_name: String,
    pub persona_name: String,
    pub display_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub has_tokens: bool,
    pub pending: bool,
    #[ts(type = "number | null")]
    pub profile_fetched_at: Option<u64>,
    pub color: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export_to = "VaultStatusDto.ts")]
pub struct VaultStatusDto {
    #[serde(flatten)]
    #[ts(flatten)]
    pub status: guard_core::authenticator::VaultStatus,
    #[ts(type = "number | null")]
    pub lock_in_seconds: Option<u64>,
}

#[derive(Serialize, TS)]
#[ts(export_to = "FolderDto.ts")]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub account_count: usize,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export_to = "SnapshotDto.ts")]
pub struct SnapshotDto {
    #[ts(type = "number")]
    pub revision: u64,
    pub accounts: Vec<AccountDto>,
    pub vault: VaultStatusDto,
    pub settings: SettingsDto,
    pub folders: Vec<FolderDto>,
    pub masked: bool,
}

#[derive(Serialize, TS)]
#[ts(export_to = "AppInfoDto.ts")]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub backup_extension: String,
    pub folder_icons: Vec<String>,
}
