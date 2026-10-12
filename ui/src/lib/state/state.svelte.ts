import { errorMessage } from "../errors/errors.svelte";
import { Language } from "../i18n/i18n.constants";
import { applyLanguage, t } from "../i18n/i18n.svelte";
import {
  SearchScope,
  SettingsSection,
  UpdateCheck,
  UpdateInterval,
  UpdateStage,
  VaultLockInterval,
} from "./state.constants";
import type { BackupMode } from "../vault/backup.constants";
import { filterAccounts } from "../accounts/search";

import { invoke, type CommandsType } from "../api/tauri";
import type { MessageType, MessagePartType } from "../i18n/messages";
import type {
  AccountType,
  AppInfoType,
  FolderType,
  SettingsType,
  SnapshotType,
  ToastAccountType,
  ToastFolderType,
  UpdateInfoType,
  VaultStatusType,
} from "../types";

export const EMPTY_SETTINGS: SettingsType = {
  language: Language.AUTO,
  resolved_language: Language.EN,
  close_to_tray: true,
  show_account_names: true,
  blur_codes: false,
  update_interval: UpdateInterval.SIX_HOURS,
  search_scope: SearchScope.BOTH,
  vault_lock_interval: VaultLockInterval.STARTUP,
  last_update_check_at: null,
};

export const EMPTY_VAULT: VaultStatusType = {
  path: "",
  lock_in_seconds: null,
  encrypted: false,
  locked: false,
  indexed: true,
  count: 0,
  error: null,
};

export type AppStateType = {
  snapshot: SnapshotType | null;
  info: AppInfoType;
  search: string;
  /** Open folder, `null` at the root. */
  folderId: string | null;
  idle: boolean;
  settingsOpen: boolean;
  settingsSection: SettingsSection;
  steamGuardOpen: boolean;
  steamGuardAccountId: string | null;
  /** Set with `steamGuardOpen` to land on the sign-in form instead of the home screen. */
  steamGuardSignIn: boolean;
  addAccountOpen: boolean;
  vaultOpen: boolean;
  vaultCopySteamId: string | null;
  backup: { mode: BackupMode; steamId: string | null } | null;
  updateDialog: boolean;
  update: UpdateInfoType | null;
  updateCheck: UpdateCheck;
  updateCheckedAt: number;
  updateStage: UpdateStage;
  updateVersion: string;
  progress: {
    downloaded: number;
    total: number;
  };
  busy: {
    active: boolean;
    text: string;
  };
  toast: {
    visible: boolean;
    text: string;
    parts: MessagePartType[];
    ok: boolean;
    account: ToastAccountType | null;
    folder: ToastFolderType | null;
  };
};

export const app: AppStateType = $state({
  snapshot: null,
  info: {
    name: "Steam Guard Desktop",
    version: "",
    repository: "",
  },
  search: "",
  folderId: null,
  idle: false,
  settingsOpen: false,
  settingsSection: SettingsSection.PREFERENCES,
  steamGuardOpen: false,
  steamGuardAccountId: null,
  steamGuardSignIn: false,
  addAccountOpen: false,
  vaultOpen: false,
  vaultCopySteamId: null,
  backup: null,
  updateDialog: false,
  update: null,
  updateCheck: UpdateCheck.IDLE,
  updateCheckedAt: 0,
  updateStage: UpdateStage.IDLE,
  updateVersion: "",
  progress: { downloaded: 0, total: 0 },
  busy: { active: false, text: t.common_working() },
  toast: { visible: false, text: "", parts: [], ok: false, account: null, folder: null },
});

let toastTimer = 0;

export const showToast = (
  content: MessageType,
  ok = false,
  account: ToastAccountType | null = null,
  folder: ToastFolderType | null = null,
): void => {
  const parts = typeof content === "string" ? [{ text: content }] : content;
  app.toast = {
    visible: true,
    text: parts.map((part) => part.text).join(""),
    parts,
    ok,
    account: account
      ? {
          avatar: account.avatar,
          initial: account.initial,
          display_name: account.display_name,
          persona_name: account.persona_name,
        }
      : null,
    folder: folder ? { name: folder.name, color: folder.color, icon: folder.icon } : null,
  };
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    app.toast = { ...app.toast, visible: false };
  }, 4200);
};

export const setBusy = (active: boolean, text: string = t.common_working()): void => {
  app.busy = { active, text };
};

export const accounts = (): AccountType[] => app.snapshot?.accounts ?? [];

export const vault = (): VaultStatusType => app.snapshot?.vault ?? EMPTY_VAULT;

export const vaultReady = (): boolean => {
  const status = vault();
  return !status.locked && !status.error;
};

export const accountById = (steamId: string): AccountType | null =>
  accounts().find((entry) => entry.steam_id === steamId) ?? null;

export const folders = (): FolderType[] => app.snapshot?.folders ?? [];

export const folderById = (id: string): FolderType | null =>
  folders().find((folder) => folder.id === id) ?? null;

export const currentFolder = (): FolderType | null =>
  app.folderId === null ? null : folderById(app.folderId);

export const visibleAccounts = (): AccountType[] =>
  filterAccounts(accounts(), {
    folderId: app.folderId,
    search: app.search,
    scope: app.snapshot?.settings.search_scope ?? SearchScope.BOTH,
  });

/** Folder cards only show at the root and outside a search. */
export const visibleFolders = (): FolderType[] =>
  app.search.trim() || app.folderId !== null ? [] : folders();

export const emptyMessage = (): string => {
  const snapshot = app.snapshot;
  if (!snapshot) {
    return "";
  }
  return snapshot.accounts.length === 0 ? t.accounts_empty() : t.accounts_search_empty();
};

export const openAccount = (steamId: string, signIn = false): void => {
  app.steamGuardSignIn = signIn;
  app.steamGuardAccountId = steamId;
  app.steamGuardOpen = true;
};

export const applySnapshot = (snapshot: SnapshotType): void => {
  if ((snapshot.revision ?? 0) < (app.snapshot?.revision ?? 0)) return;
  app.snapshot = snapshot;
  app.updateCheckedAt = Math.max(app.updateCheckedAt, snapshot.settings.last_update_check_at ?? 0);
  applyLanguage(snapshot.settings.language ?? Language.AUTO, snapshot.settings.resolved_language);
  if (app.folderId !== null && !snapshot.folders.some((folder) => folder.id === app.folderId)) {
    app.folderId = null;
  }
};

export type RunOptionsType = {
  busy?: string;
  success?: MessageType;
  account?: ToastAccountType | null;
};

export const run = async <K extends keyof CommandsType>(
  command: K,
  args?: CommandsType[K]["args"],
  options: RunOptionsType = {},
): Promise<CommandsType[K]["result"] | null> => {
  const account =
    options.account ??
    (args && typeof args === "object" && "steamId" in args && typeof args.steamId === "string"
      ? accountById(args.steamId)
      : null);
  if (options.busy) {
    setBusy(true, options.busy);
  }
  try {
    const result = await invoke(command, args);
    if (isSnapshot(result)) {
      applySnapshot(result);
    }
    if (options.success) {
      showToast(options.success, true, account);
    }
    return result;
  } catch (error) {
    showToast(errorMessage(error), false, account);
    return null;
  } finally {
    if (options.busy) {
      setBusy(false);
    }
  }
};

export const isSnapshot = (value: unknown): value is SnapshotType => {
  return (
    typeof value === "object" &&
    value !== null &&
    "accounts" in value &&
    Array.isArray(value.accounts) &&
    "vault" in value &&
    Boolean(value.vault) &&
    "settings" in value &&
    Boolean(value.settings)
  );
};
