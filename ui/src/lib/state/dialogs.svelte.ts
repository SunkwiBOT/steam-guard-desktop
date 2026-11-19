import { t } from "../i18n/i18n.svelte";
import { ConfirmKind } from "./dialogs.constants";

import type { MessageType } from "../i18n/messages";

export type ConfirmStateType = {
  open: boolean;
  title: string;
  content: MessageType;
  checkboxLabel: string;
  checkboxRequired: boolean;
  confirmLabel: string;
  danger: boolean;
  steamId: string | null;
  kind: ConfirmKind;
  busy: boolean;
};

export const confirmDialog: ConfirmStateType = $state({
  open: false,
  title: "",
  content: "",
  checkboxLabel: "",
  checkboxRequired: false,
  confirmLabel: t.common_yes(),
  danger: false,
  steamId: null,
  kind: ConfirmKind.CONFIRM,
  busy: false,
});

export type ConfirmAnswerType = {
  checked: boolean;
};

// ! Runs while the dialog stays open and busy: the action itself decides whether the dialog
// closes (`true`) or stays open for a retry (`false`), so a failed request never forces the
// user to reopen the dialog and re-tick a required checkbox.
export type ConfirmActionType = (answer: ConfirmAnswerType) => Promise<boolean>;

let confirmAction: ConfirmActionType | null = null;

export const askConfirm = ({
  title,
  text,
  checkboxLabel = "",
  checkboxRequired = false,
  confirmLabel = t.common_yes(),
  danger = false,
  steamId = null,
  kind = ConfirmKind.CONFIRM,
  action,
}: {
  title: string;
  text: MessageType;
  checkboxLabel?: string;
  checkboxRequired?: boolean;
  confirmLabel?: string;
  danger?: boolean;
  steamId?: string | null;
  kind?: ConfirmStateType["kind"];
  action: ConfirmActionType;
}): void => {
  confirmAction = action;
  Object.assign(confirmDialog, {
    open: true,
    title,
    content: text,
    checkboxLabel,
    checkboxRequired,
    confirmLabel,
    danger,
    steamId,
    kind,
    busy: false,
  });
};

export const runConfirmAction = async (answer: ConfirmAnswerType): Promise<void> => {
  if (!confirmAction || confirmDialog.busy) return;
  const action = confirmAction;
  confirmDialog.busy = true;
  try {
    if (await action(answer)) {
      confirmDialog.open = false;
      confirmAction = null;
    }
  } finally {
    confirmDialog.busy = false;
  }
};

export const cancelConfirm = (): void => {
  if (confirmDialog.busy) return;
  confirmDialog.open = false;
  confirmAction = null;
};

export type FolderDialogStateType = {
  open: boolean;
  title: string;
  value: string;
  confirmLabel: string;
};

export const folderDialog: FolderDialogStateType = $state({
  open: false,
  title: "",
  value: "",
  confirmLabel: "",
});

let folderResolve: ((value: string | null) => void) | null = null;

export const askFolderName = ({
  title = t.folder_create_title(),
  value = "",
  confirmLabel = t.common_create(),
}: { title?: string; value?: string; confirmLabel?: string } = {}): Promise<string | null> => {
  Object.assign(folderDialog, { open: true, title, value, confirmLabel });
  return new Promise((resolve) => {
    folderResolve = resolve;
  });
};

export const answerFolderName = (value: string | null): void => {
  folderDialog.open = false;
  const resolve = folderResolve;
  folderResolve = null;
  resolve?.(value);
};

export const folderEditor: { open: boolean; folderId: string } = $state({
  open: false,
  folderId: "",
});

export const openFolderEditor = (folderId: string): void => {
  Object.assign(folderEditor, { open: true, folderId });
};

export const closeFolderEditor = (): void => {
  folderEditor.open = false;
  folderEditor.folderId = "";
};
