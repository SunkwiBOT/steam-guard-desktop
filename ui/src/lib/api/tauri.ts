import type { Language } from "../i18n/i18n.constants";
import type { GuardKind } from "../steam-guard/steam-guard.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "../state/state.constants";
import type { PickKind } from "../bindings/PickKind";
import type { UrlDestination } from "../bindings/UrlDestination";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

import type { AppInfoDto } from "../bindings/AppInfoDto";
import type { BackupImportDto } from "../bindings/BackupImportDto";
import type { CodeInfo } from "../bindings/CodeInfo";
import type { ConfirmationItem } from "../bindings/ConfirmationItem";
import type { LinkStart } from "../bindings/LinkStart";
import type { LoginBegin } from "../bindings/LoginBegin";
import type { LoginResult } from "../bindings/LoginResult";
import type { LogSnapshot } from "../bindings/LogSnapshot";
import type { SettingsDto } from "../bindings/SettingsDto";
import type { SnapshotDto } from "../bindings/SnapshotDto";
import type { UpdateInfoDto } from "../bindings/UpdateInfoDto";
import type { UpdateProgressDto } from "../bindings/UpdateProgressDto";

export type UnlistenFnType = () => void;

// ! Goes through the bundled API module, never a global: a script injected
// into the page cannot reach `invoke` from `window`.
const call = (command: string, args?: Record<string, unknown>): Promise<unknown> =>
  tauriInvoke(command, args);

export type CommandsType = {
  snapshot: { args: void; result: SnapshotDto };
  refresh: { args: void; result: SnapshotDto };
  app_info: { args: void; result: AppInfoDto };
  default_settings: { args: void; result: SettingsDto };
  set_language: { args: { language: Language }; result: SnapshotDto };
  set_close_to_tray: { args: { closeToTray: boolean }; result: SnapshotDto };
  set_update_interval: { args: { interval: UpdateInterval }; result: SnapshotDto };
  set_vault_lock_interval: { args: { interval: VaultLockInterval }; result: SnapshotDto };
  set_show_account_names: { args: { visible: boolean }; result: SnapshotDto };
  set_steam_ids_visible: { args: { visible: boolean }; result: SnapshotDto };
  set_blur_codes: { args: { enabled: boolean }; result: SnapshotDto };
  set_search_scope: { args: { scope: SearchScope }; result: SnapshotDto };
  set_vault_offer_dismissed: { args: void; result: SnapshotDto };
  create_folder: {
    args: { name: string; color?: string | null; icon?: string | null };
    result: SnapshotDto;
  };
  update_folder: {
    args: { id: string; name: string; color: string | null; icon: string | null };
    result: SnapshotDto;
  };
  delete_folder: { args: { id: string }; result: SnapshotDto };
  assign_folder: { args: { steamId: string; folderId: string | null }; result: SnapshotDto };
  set_account_color: { args: { steamId: string; color: string | null }; result: SnapshotDto };
  set_folder_color: { args: { id: string; color: string | null }; result: SnapshotDto };
  set_folder_icon: { args: { id: string; icon: string | null }; result: SnapshotDto };
  logs_snapshot: { args: { previous: boolean }; result: LogSnapshot };
  logs_open_directory: { args: void; result: void };
  log_frontend_error: { args: { context: string; message: string }; result: void };
  clipboard_write_text: { args: { text: string }; result: void };
  check_for_update: { args: void; result: UpdateInfoDto | null };
  download_update: { args: void; result: string };
  frontend_ready: { args: void; result: void };
  apply_update: { args: { version: string }; result: void };
  autostart_status: { args: void; result: boolean };
  set_autostart: { args: { enabled: boolean }; result: boolean };
  authenticator_unlock: { args: { passphrase: string }; result: SnapshotDto };
  authenticator_lock: { args: void; result: SnapshotDto };
  authenticator_set_passphrase: {
    args: { passphrase: string | null; currentPassword?: string | null };
    result: SnapshotDto;
  };
  steam_guard_code: { args: { steamId: string }; result: CodeInfo };
  steam_guard_revocation_code: { args: { steamId: string }; result: string };
  account_login_name: { args: { steamId: string }; result: string };
  // ! The native open dialog runs in Rust; the picked path is kept there too,
  // only the display name reaches the page; `null` means cancelled.
  steam_guard_pick_file: { args: { kind: PickKind }; result: string | null };
  steam_guard_clear_picked_file: { args: { kind: PickKind }; result: void };
  steam_guard_import: { args: { passkey?: string | null }; result: SnapshotDto };
  // ! The destination is chosen in a native "save as" dialog opened by the
  // backend; `null` means the dialog was cancelled.
  steam_guard_export_backup: {
    args: { steamId: string | null; password: string | null };
    result: number | null;
  };
  steam_guard_import_backup: {
    args: { password: string | null };
    result: BackupImportDto;
  };
  steam_guard_confirmations: { args: { steamId: string }; result: ConfirmationItem[] };
  steam_guard_confirmation_action: {
    args: { steamId: string; id: string; nonce: string; accept: boolean };
    result: void;
  };
  steam_guard_remove: {
    args: { steamId: string; revocationCode?: string | null };
    result: SnapshotDto;
  };
  steam_guard_forget: { args: { steamId: string }; result: SnapshotDto };
  steam_guard_login_password: {
    args: { accountName: string; password: string };
    result: LoginBegin;
  };
  steam_guard_login_qr: { args: void; result: LoginBegin };
  steam_guard_login_finish: {
    args: { sessionId: string; guardKind?: GuardKind | null; code?: string | null };
    result: LoginResult;
  };
  steam_guard_login_cancel: { args: { sessionId: string }; result: void };
  steam_guard_link_start: { args: { steamId: string }; result: LinkStart };
  steam_guard_link_finalize: {
    args: { sessionId: string; confirmCode: string };
    result: SnapshotDto;
  };
  steam_guard_link_transfer_start: { args: { sessionId: string }; result: void };
  steam_guard_link_transfer_finish: {
    args: { sessionId: string; smsCode: string };
    result: SnapshotDto;
  };
  steam_guard_link_revoke: { args: { sessionId: string; revocationCode: string }; result: void };
  steam_guard_link_cancel: { args: { sessionId: string }; result: void };
  profile_refresh: { args: { steamId: string }; result: SnapshotDto };
  open_url: { args: { destination: UrlDestination }; result: void };
};

export const invoke = <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
): Promise<CommandsType[K]["result"]> => {
  return call(command, args as Record<string, unknown> | undefined) as Promise<
    CommandsType[K]["result"]
  >;
};

export type EventsType = {
  "accounts-changed": void;
  "vault-locked": void;
  "update-progress": UpdateProgressDto;
  "update-downloaded": string;
  "clock-offset-measured": number;
  "clock-drift-warning": number;
};

export const listen = <K extends keyof EventsType>(
  name: K,
  handler: (event: { payload: EventsType[K] }) => void,
): Promise<UnlistenFnType> => {
  return tauriListen(name, handler as (event: { payload: unknown }) => void);
};
