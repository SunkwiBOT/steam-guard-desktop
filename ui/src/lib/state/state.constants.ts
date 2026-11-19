export enum SettingsSection {
  PREFERENCES = "preferences",
  FOLDERS = "folders",
  VAULT = "vault",
  LOGS = "logs",
  UPDATES = "updates",
}

export enum UpdateCheck {
  IDLE = "idle",
  CHECKING = "checking",
  SUCCESS = "success",
  ERROR = "error",
}

export { UpdateInterval } from "../bindings/UpdateInterval";
export { SearchScope } from "../bindings/SearchScope";
export { VaultLockInterval } from "../bindings/VaultLockInterval";

export enum UpdateStage {
  IDLE = "idle",
  DOWNLOADING = "downloading",
  DOWNLOADED = "downloaded",
  INSTALLING = "installing",
}

export enum ToastKind {
  INFO = "INFO",
  SUCCESS = "SUCCESS",
  ERROR = "ERROR",
}

export const TOAST_DURATION_MS: Record<ToastKind, number> = {
  [ToastKind.INFO]: 4_200,
  [ToastKind.SUCCESS]: 4_200,
  [ToastKind.ERROR]: 8_000,
};
