import { errorMessage } from "../errors/errors.svelte";
import { t, rich, locale } from "../i18n/i18n.svelte";
import { ConfirmKind } from "../state/dialogs.constants";
import { UpdateCheck, UpdateInterval, UpdateStage } from "../state/state.constants";
import { BackupMode } from "../vault/backup.constants";
import { updateCheckDue } from "../updates/update-policy";

import { randomFolderStyle } from "../folders/folder-style";
import { copyValue } from "../state/clipboard.svelte";
import { askConfirm, askFolderName, confirmDialog, folderDialog } from "../state/dialogs.svelte";
import {
  accountById,
  app,
  folderById,
  folders,
  openAccount,
  run,
  showToast,
  vault,
} from "../state/state.svelte";
import { invoke, invokePlugin } from "../api/tauri";

const FALLBACK_REPOSITORY_URL = "https://github.com/SunkwiBOT/steam-guard-desktop";

export const repositoryUrl = (): string => app.info.repository || FALLBACK_REPOSITORY_URL;

export const openRepository = async (): Promise<void> => {
  const url = repositoryUrl();
  try {
    await invokePlugin("plugin:opener|open_url", { url, with: null });
  } catch (error) {
    showToast(errorMessage(error));
  }
};

/** Fetch a fresh code at click time, rather than copying a stale display. */
export const copySteamGuardCode = async (steamId: string): Promise<boolean> => {
  if (app.busy.active) return false;
  const entry = accountById(steamId);
  if (vault().locked) {
    app.vaultCopySteamId = steamId;
    app.vaultOpen = true;
    return false;
  }
  try {
    const { code } = await invoke("steam_guard_code", { steamId });
    return await copyValue(code, `guard:${steamId}`, t.guard_code_label(), entry);
  } catch (cause) {
    showToast(errorMessage(cause), false, entry);
    return false;
  }
};

export const forgetAccount = async (steamId: string): Promise<boolean> => {
  if (app.busy.active || confirmDialog.open) return false;
  const entry = accountById(steamId);
  if (!entry) return false;
  const answer = await askConfirm({
    title: t.guard_forget_title(),
    kind: ConfirmKind.REMOVE,
    steamId,
    text: rich(t.guard_forget_warning, { account: entry.display_name }),
    checkboxLabel: t.guard_forget_acknowledgement(),
    checkboxRequired: true,
    confirmLabel: t.guard_forget_action(),
    danger: true,
  });
  if (!answer?.checked) return false;
  const result = await run(
    "steam_guard_forget",
    { steamId, confirmed: true },
    {
      busy: t.guard_forget_pending(),
      success: rich(t.guard_forget_success, { account: entry.display_name }),
      account: entry,
    },
  );
  return result !== null;
};

export const refreshProfile = async (steamId: string): Promise<boolean> => {
  const entry = accountById(steamId);
  if (!entry) return false;
  const result = await run(
    "profile_refresh",
    { steamId },
    { success: t.profile_refresh_success(), account: entry },
  );
  return result !== null;
};

export const exportAccount = (steamId: string): void => {
  app.backup = { mode: BackupMode.EXPORT, steamId };
};

/** Opens the account on its sign-in form; a pending link resumes from the home screen instead. */
export const signInAgain = (steamId: string): void => {
  const entry = accountById(steamId);
  if (!entry) return;
  openAccount(steamId, !entry.pending);
};

export const moveToFolder = async (steamId: string, folderId: string | null): Promise<boolean> => {
  const entry = accountById(steamId);
  const folder = folderId ? folderById(folderId) : null;
  if (!entry || entry.folder_id === folderId) return false;
  const result = await run(
    "assign_folder",
    { steamId, folderId },
    folder ? { success: rich(t.folder_move_success, { folder: folder.name }), account: entry } : {},
  );
  return result !== null;
};

export const setAccountColor = async (steamId: string, color: string): Promise<boolean> => {
  const result = await run("set_account_color", { steamId, color: color || null });
  return result !== null;
};

/** Creates a folder with a random style; returns the new folder's id. */
export const createFolder = async (name: string): Promise<string | null> => {
  const style = randomFolderStyle();
  //! The new id is the one that was not there before, never the first folder
  //! sharing the name.
  const known = new Set(folders().map((item) => item.id));
  const result = await run("create_folder", { name, color: style.color, icon: style.icon });
  if (!result) return null;
  const created = folders().find((item) => !known.has(item.id));
  if (created) showToast(rich(t.folder_create_success, { folder: name }), true, null, created);
  return created?.id ?? null;
};

/** Asks for a name, creates the folder and moves the account into it. */
export const createFolderFor = async (steamId: string): Promise<void> => {
  if (app.busy.active || folderDialog.open) return;
  const name = await askFolderName();
  if (!name) return;
  const folderId = await createFolder(name);
  if (folderId) await moveToFolder(steamId, folderId);
};

export const deleteFolder = async (id: string): Promise<boolean> => {
  const folder = folderById(id);
  if (!folder || app.busy.active || confirmDialog.open) return false;
  const answer = await askConfirm({
    title: t.folder_delete_title(),
    text: rich(t.folder_delete_confirm, { folder: folder.name }),
    confirmLabel: t.common_delete(),
    danger: true,
    kind: ConfirmKind.REMOVE,
  });
  if (!answer) return false;
  const result = await run("delete_folder", { id });
  if (result === null) return false;
  showToast(rich(t.folder_delete_success, { folder: folder.name }), true, null, folder);
  return true;
};

export const checkForUpdate = async (): Promise<void> => {
  if (app.updateStage || app.updateCheck === UpdateCheck.CHECKING) {
    return;
  }
  app.updateCheck = UpdateCheck.CHECKING;
  try {
    app.update = await invoke("check_for_update");
    app.updateCheck = UpdateCheck.SUCCESS;
  } catch {
    app.update = null;
    app.updateCheck = UpdateCheck.ERROR;
  } finally {
    app.updateCheckedAt = Math.floor(Date.now() / 1000);
  }
};

export const checkForUpdateIfDue = async (startup = false): Promise<void> => {
  const interval = app.snapshot?.settings.update_interval ?? UpdateInterval.SIX_HOURS;
  if (updateCheckDue(interval, app.updateCheckedAt, Math.floor(Date.now() / 1000), startup)) {
    await checkForUpdate();
  }
};

export const updateButtonLabel = (): string => {
  if (app.updateStage === UpdateStage.INSTALLING) return t.common_restarting();
  if (app.updateStage === UpdateStage.DOWNLOADED) {
    return t.common_restart();
  }
  if (app.updateStage === UpdateStage.DOWNLOADING) {
    return t.common_downloading();
  }
  return app.update ? t.updates_action({ version: app.update.version }) : "";
};

export const downloadUpdate = async (): Promise<void> => {
  if (app.updateStage) return;
  // The progress dialog replaces the release notes; "Later" hides it and the
  // header button brings it back.
  app.updateDialog = true;
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: 0, total: 0 };

  const version = await run("download_update");
  if (typeof version !== "string") {
    app.updateStage = UpdateStage.IDLE;
    return;
  }
  // The download always takes the newest release, which can differ from the one checked earlier.
  if (app.update && app.update.version !== version) {
    app.update = { ...app.update, version };
  }
  markDownloaded(version);
};

export const markDownloaded = (version: string): void => {
  if (app.updateStage === UpdateStage.INSTALLING) return;
  app.updateStage = UpdateStage.DOWNLOADED;
  app.updateVersion = version;
  app.updateDialog = true;
};

export const recordUpdateProgress = (payload: {
  downloaded?: number;
  total?: number | null;
}): void => {
  if (app.updateStage === UpdateStage.DOWNLOADED || app.updateStage === UpdateStage.INSTALLING) {
    return;
  }
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: payload.downloaded ?? 0, total: payload.total ?? 0 };
};

export const applyUpdate = async (): Promise<void> => {
  if (app.updateStage !== UpdateStage.DOWNLOADED) return;
  app.updateStage = UpdateStage.INSTALLING;
  try {
    await invoke("apply_update", { version: app.updateVersion });
  } catch (cause) {
    app.updateStage = UpdateStage.DOWNLOADED;
    showToast(errorMessage(cause));
  }
};

const megabytes = (bytes: number): string => {
  return t.updates_size_megabytes({
    size: new Intl.NumberFormat(locale.current, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }).format(bytes / 1024 / 1024),
  });
};

export const updateProgressText = (): string => {
  const { downloaded, total } = app.progress;
  if (!total) {
    return t.updates_progress_downloaded({ downloaded: megabytes(downloaded) });
  }
  const percent = Math.min(100, Math.round((downloaded / total) * 100));
  return t.updates_progress_total({
    percent,
    downloaded: megabytes(downloaded),
    total: megabytes(total),
  });
};

export const updateProgressPercent = (): number => {
  const { downloaded, total } = app.progress;
  return total ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
};
