import { errorMessage } from "../errors/errors.svelte";
import { t, rich, locale } from "../i18n/i18n.svelte";
import { ConfirmKind } from "../state/dialogs.constants";
import { UpdateCheck, UpdateStage, UpdateInterval, ToastKind } from "../state/state.constants";
import { BackupMode } from "../vault/backup.constants";
import { UrlDestination } from "../bindings/UrlDestination";
import { updateCheckDue } from "../updates/update-policy";

import { randomFolderStyle } from "../folders/folder-style";
import { copyValue } from "../state/clipboard.svelte";
import { askConfirm, askFolderName, confirmDialog, folderDialog } from "../state/dialogs.svelte";
import { toast } from "../state/toasts";
import {
  accountById,
  app,
  folderById,
  folders,
  openAccount,
  run,
  vault,
} from "../state/state.svelte";
import { invoke } from "../api/tauri";

export const repositoryUrl = (): string => app.info.repository;

export const openRepository = async (): Promise<void> => {
  try {
    await invoke("open_url", { destination: UrlDestination.PROJECT_PAGE });
  } catch (e: unknown) {
    toast(ToastKind.ERROR, t.action_failed_title(), { description: errorMessage(e) });
  }
};

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
  } catch (e: unknown) {
    toast(ToastKind.ERROR, t.action_failed_title(), {
      description: errorMessage(e),
      account: entry,
    });
    return false;
  }
};

export const forgetAccount = (steamId: string, onForgotten?: () => void): void => {
  if (app.busy.active || confirmDialog.open) return;
  const entry = accountById(steamId);
  if (!entry) return;
  askConfirm({
    title: t.guard_forget_title(),
    kind: ConfirmKind.REMOVE,
    steamId,
    text: rich(t.guard_forget_warning, { account: entry.display_name }),
    checkboxLabel: t.guard_forget_acknowledgement(),
    checkboxRequired: true,
    confirmLabel: t.guard_forget_action(),
    danger: true,
    action: async (answer) => {
      if (!answer.checked) return false;
      const result = await run(
        "steam_guard_forget",
        { steamId },
        {
          success: {
            title: t.account_forgotten_title(),
            description: rich(t.guard_forget_success, { account: entry.display_name }),
          },
          account: entry,
        },
      );
      if (result === null) return false;
      onForgotten?.();
      return true;
    },
  });
};

export const refreshProfile = async (steamId: string): Promise<boolean> => {
  const entry = accountById(steamId);
  if (!entry) return false;
  const result = await run(
    "profile_refresh",
    { steamId },
    { success: { title: t.profile_refresh_success() }, account: entry },
  );
  return result !== null;
};

export const exportAccount = (steamId: string): void => {
  app.backup = { mode: BackupMode.EXPORT, steamId };
};

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
    folder
      ? {
          success: {
            title: t.folder_moved_title(),
            description: rich(t.folder_move_success, { folder: folder.name }),
          },
          account: entry,
        }
      : {},
  );
  return result !== null;
};

export const setAccountColor = async (steamId: string, color: string): Promise<boolean> => {
  const result = await run("set_account_color", { steamId, color: color || null });
  return result !== null;
};

export const createFolder = async (name: string): Promise<string | null> => {
  const style = randomFolderStyle(app.info.folder_icons);
  const known = new Set(folders().map((item) => item.id));
  const result = await run("create_folder", { name, color: style.color, icon: style.icon });
  if (!result) return null;
  const created = folders().find((item) => !known.has(item.id));
  if (created)
    toast(ToastKind.SUCCESS, t.folder_created_title(), {
      description: rich(t.folder_create_success, { folder: name }),
      folder: created,
    });
  return created?.id ?? null;
};

export const createFolderFor = async (steamId: string): Promise<void> => {
  if (app.busy.active || folderDialog.open) return;
  const name = await askFolderName();
  if (!name) return;
  const folderId = await createFolder(name);
  if (folderId) await moveToFolder(steamId, folderId);
};

export const deleteFolder = (id: string): void => {
  const folder = folderById(id);
  if (!folder || app.busy.active || confirmDialog.open) return;
  askConfirm({
    title: t.folder_delete_title(),
    text: rich(t.folder_delete_confirm, { folder: folder.name }),
    confirmLabel: t.common_delete(),
    danger: true,
    kind: ConfirmKind.REMOVE,
    action: async () => {
      const result = await run("delete_folder", { id });
      if (result === null) return false;
      toast(ToastKind.SUCCESS, t.folder_deleted_title(), {
        description: rich(t.folder_delete_success, { folder: folder.name }),
        folder,
      });
      return true;
    },
  });
};

export const checkForUpdate = async (): Promise<void> => {
  if (app.updateStage !== UpdateStage.IDLE || app.updateCheck === UpdateCheck.CHECKING) {
    return;
  }
  app.updateCheck = UpdateCheck.CHECKING;
  try {
    app.update = await invoke("check_for_update");
    app.updateCheck = UpdateCheck.SUCCESS;
  } catch (e: unknown) {
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
  if (app.updateStage !== UpdateStage.IDLE) return;
  app.updateDialog = true;
  app.updateStage = UpdateStage.DOWNLOADING;
  app.progress = { downloaded: 0, total: 0 };

  const version = await run("download_update");
  if (typeof version !== "string") {
    app.updateStage = UpdateStage.IDLE;
    return;
  }
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
  } catch (e: unknown) {
    app.updateStage = UpdateStage.DOWNLOADED;
    toast(ToastKind.ERROR, t.action_failed_title(), { description: errorMessage(e) });
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
