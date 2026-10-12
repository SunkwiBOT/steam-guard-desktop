import type { Language } from "../i18n/i18n.constants";
import type { GuardKind } from "../steam-guard/steam-guard.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "../state/state.constants";
import type { PickFileKind } from "../vault/backup.constants";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

import type {
  AppInfoType,
  BackupImportType,
  CodeInfoType,
  ConfirmationItemType,
  LinkStartType,
  LoginBeginType,
  LoginResultType,
  LogSnapshotType,
  SnapshotType,
  StoredAuthenticatorType,
  UpdateInfoType,
  UpdateProgressType,
  VaultStatusType,
} from "../types";

export type UnlistenFnType = () => void;

//! Goes through the bundled API module, never a global: a script injected
//! into the page cannot reach `invoke` from `window`.
const call = (command: string, args?: Record<string, unknown>): Promise<unknown> =>
  tauriInvoke(command, args);

export type CommandsType = {
  snapshot: { args: void; result: SnapshotType };
  refresh: { args: void; result: SnapshotType };
  app_info: { args: void; result: AppInfoType };
  set_language: { args: { language: Language }; result: SnapshotType };
  set_close_to_tray: { args: { closeToTray: boolean }; result: SnapshotType };
  set_update_interval: { args: { interval: UpdateInterval }; result: SnapshotType };
  set_vault_lock_interval: { args: { interval: VaultLockInterval }; result: SnapshotType };
  set_show_account_names: { args: { visible: boolean }; result: SnapshotType };
  set_blur_codes: { args: { enabled: boolean }; result: SnapshotType };
  set_search_scope: { args: { scope: SearchScope }; result: SnapshotType };
  create_folder: {
    args: { name: string; color?: string | null; icon?: string | null };
    result: SnapshotType;
  };
  update_folder: {
    args: { id: string; name: string; color: string | null; icon: string | null };
    result: SnapshotType;
  };
  delete_folder: { args: { id: string }; result: SnapshotType };
  assign_folder: { args: { steamId: string; folderId: string | null }; result: SnapshotType };
  set_account_color: { args: { steamId: string; color: string | null }; result: SnapshotType };
  set_folder_color: { args: { id: string; color: string | null }; result: SnapshotType };
  set_folder_icon: { args: { id: string; icon: string | null }; result: SnapshotType };
  set_window_active: { args: { active: boolean }; result: void };
  logs_snapshot: { args: { previous: boolean }; result: LogSnapshotType };
  logs_open_directory: { args: void; result: void };
  clipboard_write_text: { args: { text: string }; result: void };
  check_for_update: { args: void; result: UpdateInfoType | null };
  download_update: { args: void; result: string };
  apply_update: { args: { version: string }; result: void };
  autostart_status: { args: void; result: boolean };
  set_autostart: { args: { enabled: boolean }; result: boolean };
  authenticator_status: { args: void; result: VaultStatusType };
  authenticator_accounts: { args: void; result: StoredAuthenticatorType[] };
  authenticator_unlock: { args: { passphrase: string }; result: SnapshotType };
  authenticator_lock: { args: void; result: SnapshotType };
  authenticator_set_passphrase: {
    args: { passphrase: string | null; currentPassword?: string | null };
    result: SnapshotType;
  };
  steam_guard_code: { args: { steamId: string }; result: CodeInfoType };
  steam_guard_revocation_code: { args: { steamId: string }; result: string };
  /** The native open dialog runs in Rust; `null` means it was cancelled. */
  steam_guard_pick_file: { args: { kind: PickFileKind }; result: string | null };
  steam_guard_import: { args: { path: string; passkey?: string | null }; result: SnapshotType };
  /** The destination is chosen in a native "save as" dialog opened by the
   *  backend; `null` means the dialog was cancelled. */
  steam_guard_export_backup: {
    args: { steamId: string | null; password: string | null };
    result: number | null;
  };
  steam_guard_import_backup: {
    args: { path: string; password: string | null };
    result: BackupImportType;
  };
  steam_guard_confirmations: { args: { steamId: string }; result: ConfirmationItemType[] };
  steam_guard_confirmation_action: {
    args: { steamId: string; id: string; nonce: string; accept: boolean };
    result: void;
  };
  steam_guard_remove: {
    args: { steamId: string; revocationCode?: string | null };
    result: SnapshotType;
  };
  steam_guard_forget: { args: { steamId: string; confirmed: boolean }; result: SnapshotType };
  steam_guard_login_password: {
    args: { accountName: string; password: string };
    result: LoginBeginType;
  };
  steam_guard_login_qr: { args: void; result: LoginBeginType };
  steam_guard_login_finish: {
    args: { sessionId: string; guardKind?: GuardKind | null; code?: string | null };
    result: LoginResultType;
  };
  steam_guard_login_cancel: { args: { sessionId: string }; result: void };
  steam_guard_link_start: { args: { steamId: string }; result: LinkStartType };
  steam_guard_link_finalize: {
    args: { sessionId: string; confirmCode: string };
    result: SnapshotType;
  };
  steam_guard_link_transfer_start: { args: { sessionId: string }; result: void };
  steam_guard_link_transfer_finish: {
    args: { sessionId: string; smsCode: string };
    result: SnapshotType;
  };
  steam_guard_link_revoke: { args: { sessionId: string; revocationCode: string }; result: void };
  steam_guard_link_cancel: { args: { sessionId: string }; result: void };
  profile_refresh: { args: { steamId: string }; result: SnapshotType };
};

export const invoke = <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
): Promise<CommandsType[K]["result"]> => {
  return call(command, args as Record<string, unknown> | undefined) as Promise<
    CommandsType[K]["result"]
  >;
};

/** Calls a plugin command: the typed `invoke` above only covers the commands
 *  the application itself registers. */
export const invokePlugin = (command: string, args?: Record<string, unknown>): Promise<unknown> => {
  return call(command, args);
};

export type EventsType = {
  "accounts-changed": void;
  "vault-locked": void;
  "update-progress": UpdateProgressType;
  "update-downloaded": string;
};

export const listen = <K extends keyof EventsType>(
  name: K,
  handler: (event: { payload: EventsType[K] }) => void,
): Promise<UnlistenFnType> => {
  return tauriListen(name, handler as (event: { payload: unknown }) => void);
};
