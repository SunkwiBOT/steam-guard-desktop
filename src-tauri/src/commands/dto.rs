use serde::Serialize;

#[derive(Serialize)]
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
    pub last_update_check_at: Option<u64>,
}

#[derive(Clone, Serialize)]
pub struct AccountDto {
    pub steam_id: String,
    /// Empty while the vault is locked or the names are masked.
    pub account_name: String,
    pub persona_name: String,
    pub display_name: String,
    pub initial: String,
    pub avatar: Option<String>,
    pub has_tokens: bool,
    /// Link started, not finalized.
    pub pending: bool,
    pub profile_fetched_at: Option<u64>,
    pub color: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Serialize)]
pub struct VaultStatusDto {
    #[serde(flatten)]
    pub status: guard_core::authenticator::VaultStatus,
    pub lock_in_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct FolderDto {
    pub id: String,
    pub name: String,
    pub account_count: usize,
    /// ! Always false here: this app has no read-only folder. Kept so the
    /// interface shares the switcher's contract.
    pub system: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize)]
pub struct SnapshotDto {
    pub revision: u64,
    pub accounts: Vec<AccountDto>,
    pub vault: VaultStatusDto,
    pub settings: SettingsDto,
    pub folders: Vec<FolderDto>,
    pub masked: bool,
}

#[derive(Serialize)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub repository: String,
}
