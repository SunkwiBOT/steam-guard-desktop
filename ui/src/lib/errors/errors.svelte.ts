import { UiErrorCode, isVaultRequiresNewerVersion, resolveErrorCode } from "./errors.constants";
import { t, rich } from "../i18n/i18n.svelte";
import type { MessageType } from "../i18n/messages";

export { isVaultRequiresNewerVersion, resolveErrorCode };

const messages: Record<UiErrorCode, () => MessageType> = {
  [UiErrorCode.PASSWORD_MISMATCH]: () => t.error_password_mismatch(),
  [UiErrorCode.BACKUP_FILE_REQUIRED]: () => t.error_backup_file_required(),
  [UiErrorCode.STEAM_PASSWORD_REQUIRED]: () => t.error_steam_password_required(),
  [UiErrorCode.ACCOUNT_NAME_REQUIRED]: () => t.error_account_name_required(),
  [UiErrorCode.GUARD_CODE_REQUIRED]: () => t.error_guard_code_required(),
  [UiErrorCode.REVOCATION_CODE_REQUIRED]: () => t.error_revocation_code_required(),

  [UiErrorCode.UNKNOWN_ERROR]: () => t.error_unknown_error(),
  [UiErrorCode.BACKUP_EMPTY]: () => t.error_backup_empty(),
  [UiErrorCode.BACKUP_EXPORT_FAILED]: () => t.error_backup_export_failed(),
  [UiErrorCode.BACKUP_TARGET_UNSAFE]: () => t.error_backup_target_unsafe(),
  [UiErrorCode.UNSUPPORTED_BACKUP_FORMAT]: () => t.error_unsupported_backup_format(),
  [UiErrorCode.VAULT_LOCKED]: () => t.error_vault_locked(),
  [UiErrorCode.AUTHENTICATOR_MISSING]: () => t.error_authenticator_missing(),
  [UiErrorCode.WRONG_VAULT_PASSWORD]: () => t.error_wrong_vault_password(),
  [UiErrorCode.INVALID_VAULT]: () => t.error_invalid_vault(),
  [UiErrorCode.INVALID_BACKUP]: () => t.error_invalid_backup(),
  [UiErrorCode.INVALID_BACKUP_MANIFEST]: () =>
    rich(t.error_invalid_backup_manifest, { manifest: "manifest.json" }, ["manifest"]),
  [UiErrorCode.BACKUP_DECRYPTION_FAILED]: () => t.error_backup_decryption_failed(),
  [UiErrorCode.INVALID_AUTHENTICATOR_DATA]: () => t.error_invalid_authenticator_data(),
  [UiErrorCode.SESSION_EXPIRED]: () => t.error_session_expired(),
  [UiErrorCode.SIGN_IN_REQUIRED]: () => t.error_sign_in_required(),
  [UiErrorCode.AUTHENTICATOR_BUSY]: () => t.error_authenticator_busy(),
  [UiErrorCode.STEAM_ERROR]: () => t.error_steam_error(),
  [UiErrorCode.NETWORK_ERROR]: () => t.error_network_error(),
  [UiErrorCode.IO_ERROR]: () => t.error_io_error(),
  [UiErrorCode.FILE_PERMISSION]: () => t.error_file_permission(),
  [UiErrorCode.INVALID_STEAM_ID]: () => t.error_invalid_steam_id(),
  [UiErrorCode.QR_CODE_FAILED]: () => t.error_qr_code_failed(),
  [UiErrorCode.LINK_NOT_STARTED]: () => t.error_link_not_started(),
  [UiErrorCode.LINK_FAILED]: () => t.error_link_failed(),
  [UiErrorCode.LINK_NOT_CONFIRMED]: () => t.error_link_not_confirmed(),
  [UiErrorCode.TRANSFER_START_FAILED]: () => t.error_transfer_start_failed(),
  [UiErrorCode.TRANSFER_NOT_STARTED]: () => t.error_transfer_not_started(),
  [UiErrorCode.TRANSFER_FAILED]: () => t.error_transfer_failed(),
  [UiErrorCode.VAULT_PASSWORD_REQUIRED_TO_LOCK]: () => t.error_vault_password_required_to_lock(),
  [UiErrorCode.ACCOUNT_MISMATCH]: () => t.error_account_mismatch(),
  [UiErrorCode.AUTHENTICATOR_ALREADY_SAVED]: () => t.error_authenticator_already_saved(),
  [UiErrorCode.ACTIVATION_REQUIRED]: () => t.error_activation_required(),
  [UiErrorCode.VAULT_ENCRYPTION_FAILED]: () => t.error_vault_encryption_failed(),
  [UiErrorCode.VAULT_OPEN_FAILED]: () => t.error_vault_open_failed(),
  [UiErrorCode.UNSUPPORTED_VAULT_FORMAT]: () => t.error_unsupported_vault_format(),
  [UiErrorCode.UNSUPPORTED_VAULT_PROTECTION]: () => t.error_unsupported_vault_protection(),
  [UiErrorCode.CURRENT_PASSWORD_REQUIRED]: () => t.error_current_password_required(),
  [UiErrorCode.VAULT_PASSWORD_REQUIRED]: () => t.error_vault_password_required(),
  [UiErrorCode.BACKUP_PASSWORD_REQUIRED]: () => t.error_backup_password_required(),
  [UiErrorCode.BACKUP_MANIFEST_MISSING]: () =>
    rich(t.error_backup_manifest_missing, { manifest: "manifest.json" }, ["manifest"]),
  [UiErrorCode.BACKUP_MANIFEST_MISMATCH]: () =>
    rich(t.error_backup_manifest_mismatch, { manifest: "manifest.json" }, ["manifest"]),
  [UiErrorCode.BACKUP_STEAM_ID_MISSING]: () => t.error_backup_steam_id_missing(),
  [UiErrorCode.BACKUP_DEVICE_MISSING]: () => t.error_backup_device_missing(),
  [UiErrorCode.CONFIRMATION_REQUIRED]: () => t.error_confirmation_required(),
  [UiErrorCode.ACCOUNT_UNAVAILABLE]: () => t.error_account_unavailable(),
  [UiErrorCode.CLIPBOARD_UNAVAILABLE]: () => t.error_clipboard_unavailable(),
  [UiErrorCode.LOG_UNAVAILABLE]: () => t.error_log_unavailable(),
  [UiErrorCode.OPEN_FOLDER_FAILED]: () => t.error_open_folder_failed(),
  [UiErrorCode.OPEN_URL_FAILED]: () => t.error_open_url_failed(),
  [UiErrorCode.UPDATE_FAILED]: () => t.error_update_failed(),
  [UiErrorCode.NO_UPDATE]: () => t.error_no_update(),
  [UiErrorCode.NO_DOWNLOADED_UPDATE]: () => t.error_no_downloaded_update(),
  [UiErrorCode.UPDATE_PORTABLE_UNSUPPORTED]: () => t.error_update_portable_unsupported(),
  [UiErrorCode.WORKER_FAILED]: () => t.error_worker_failed(),
  [UiErrorCode.SETTINGS_SAVE_FAILED]: () => t.error_settings_save_failed(),
  [UiErrorCode.AUTOSTART_FAILED]: () => t.error_autostart_failed(),
  [UiErrorCode.INVALID_LANGUAGE]: () => t.error_invalid_language(),
  [UiErrorCode.WRONG_STEAM_PASSWORD]: () => t.error_wrong_steam_password(),
  [UiErrorCode.WRONG_GUARD_CODE]: () => t.error_wrong_guard_code(),
  [UiErrorCode.STEAM_RATE_LIMITED]: () => t.error_steam_rate_limited(),
  [UiErrorCode.DATABASE_VERSION_MISMATCH]: () => t.error_database_version_mismatch(),
  [UiErrorCode.VAULT_REQUIRES_NEWER_VERSION]: () => t.error_vault_requires_newer_version(),
  [UiErrorCode.PROFILE_UNAVAILABLE]: () => t.error_profile_unavailable(),
  [UiErrorCode.FOLDER_NAME_REQUIRED]: () => t.error_folder_name_required(),
  [UiErrorCode.UNKNOWN_FOLDER]: () => t.error_unknown_folder(),
  [UiErrorCode.LINK_IN_FLIGHT]: () => t.error_link_in_flight(),
};

export const vaultErrorHint = (value: unknown): MessageType =>
  isVaultRequiresNewerVersion(value)
    ? t.vault_requires_newer_version_hint()
    : t.vault_restore_hint();

export const errorMessage = (value: unknown): MessageType => messages[resolveErrorCode(value)]();
