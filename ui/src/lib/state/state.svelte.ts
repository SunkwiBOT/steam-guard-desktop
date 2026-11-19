import { errorMessage } from "../errors/errors.svelte";
import { type Language, type ResolvedLanguageType } from "../i18n/i18n.constants";
import { applyLanguage, t } from "../i18n/i18n.svelte";
import {
  SettingsSection,
  UpdateCheck,
  UpdateStage,
  SearchScope,
  ToastKind,
} from "./state.constants";
import type { BackupMode } from "../vault/backup.constants";
import { filterAccounts } from "../accounts/search";

import { invoke, type CommandsType } from "../api/tauri";
import type { MessageType } from "../i18n/messages";
import type { AccountDto } from "../bindings/AccountDto";
import type { AppInfoDto } from "../bindings/AppInfoDto";
import type { FolderDto } from "../bindings/FolderDto";
import type { SettingsDto } from "../bindings/SettingsDto";
import type { SnapshotDto } from "../bindings/SnapshotDto";
import type { UpdateInfoDto } from "../bindings/UpdateInfoDto";
import type { VaultStatusDto } from "../bindings/VaultStatusDto";
import type { ToastAccountType, ToastFolderType } from "../types";
import { toast } from "./toasts";

// ! Placeholder settings shown before `default_settings`/`snapshot` resolve at startup; `loadDefaultSettings`
// fetches the real value from `Settings::default()` (src-tauri/src/settings/model.rs) instead of mirroring it.
export const defaultSettingsState: { value: SettingsDto | null } = $state({ value: null });

export const loadDefaultSettings = async (): Promise<void> => {
  defaultSettingsState.value = await invoke("default_settings");
};

export const EMPTY_VAULT: VaultStatusDto = {
  path: "",
  lock_in_seconds: null,
  encrypted: false,
  locked: false,
  indexed: true,
  count: 0,
  error: null,
  password_suggested: false,
};

export type AppStateType = {
  snapshot: SnapshotDto | null;
  info: AppInfoDto;
  search: string;
  folderId: string | null;
  idle: boolean;
  settingsOpen: boolean;
  settingsSection: SettingsSection;
  steamGuardOpen: boolean;
  steamGuardAccountId: string | null;
  steamGuardSignIn: boolean;
  addAccountOpen: boolean;
  vaultOpen: boolean;
  vaultOfferOpen: boolean;
  vaultCopySteamId: string | null;
  backup: { mode: BackupMode; steamId: string | null } | null;
  updateDialog: boolean;
  update: UpdateInfoDto | null;
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
};

export const app: AppStateType = $state({
  snapshot: null,
  info: {
    name: "Steam Guard Desktop",
    version: "",
    repository: "",
    backup_extension: "",
    folder_icons: [],
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
  vaultOfferOpen: false,
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
});

export const setBusy = (active: boolean, text: string = t.common_working()): void => {
  app.busy = { active, text };
};

export const accounts = (): AccountDto[] => app.snapshot?.accounts ?? [];

export const vault = (): VaultStatusDto => app.snapshot?.vault ?? EMPTY_VAULT;

export const vaultReady = (): boolean => {
  const status = vault();
  return !status.locked && !status.error;
};

export const accountById = (steamId: string): AccountDto | null =>
  accounts().find((entry) => entry.steam_id === steamId) ?? null;

export const folders = (): FolderDto[] => app.snapshot?.folders ?? [];

export const folderById = (id: string): FolderDto | null =>
  folders().find((folder) => folder.id === id) ?? null;

export const currentFolder = (): FolderDto | null =>
  app.folderId === null ? null : folderById(app.folderId);

export const visibleAccounts = (): AccountDto[] =>
  filterAccounts(accounts(), {
    folderId: app.folderId,
    search: app.search,
    scope: app.snapshot?.settings.search_scope ?? SearchScope.BOTH,
  });

export const visibleFolders = (): FolderDto[] =>
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

export const applySnapshot = (snapshot: SnapshotDto): void => {
  if (snapshot.revision < (app.snapshot?.revision ?? 0)) return;
  app.snapshot = snapshot;
  app.updateCheckedAt = Math.max(app.updateCheckedAt, snapshot.settings.last_update_check_at ?? 0);
  // ! Wire values from the backend: plain strings restricted to `Language`/`ResolvedLanguageType` members.
  applyLanguage(
    snapshot.settings.language as Language,
    snapshot.settings.resolved_language as ResolvedLanguageType,
  );
  if (app.folderId !== null && !snapshot.folders.some((folder) => folder.id === app.folderId)) {
    app.folderId = null;
  }
};

export type RunOptionsType = {
  busy?: string;
  success?: { title: string; description?: MessageType } | null;
  account?: ToastAccountType | null;
  folder?: ToastFolderType | null;
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
      toast(ToastKind.SUCCESS, options.success.title, {
        description: options.success.description,
        account,
        folder: options.folder,
      });
    }
    return result;
  } catch (e: unknown) {
    toast(ToastKind.ERROR, t.action_failed_title(), {
      description: errorMessage(e),
      account,
      folder: options.folder,
    });
    return null;
  } finally {
    if (options.busy) {
      setBusy(false);
    }
  }
};

export const isSnapshot = (value: unknown): value is SnapshotDto => {
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
