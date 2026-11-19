export enum GuardKind {
  DEVICE_CODE = "DEVICE_CODE",
  EMAIL_CODE = "EMAIL_CODE",
  DEVICE_CONFIRMATION = "DEVICE_CONFIRMATION",
  EMAIL_CONFIRMATION = "EMAIL_CONFIRMATION",
  NONE = "NONE",
  UNKNOWN = "UNKNOWN",
}

export enum SteamGuardScreen {
  HOME = "home",
  IMPORT = "import",
  LOGIN = "login",
  LINK = "link",
}

export enum LoginPurpose {
  LINK = "link",
  SIGN_IN = "signin",
}

export enum GuardAction {
  IDLE = "",
  ACTION = "action",
  PASSWORD = "password",
  QR = "qr",
  LOGIN = "login",
  LOGIN_CODE = "login-code",
  LINK = "link",
  CANCEL_LOGIN = "cancel-login",
  CANCEL_LINK = "cancel-link",
  BROWSE = "browse",
  IMPORT = "import",
  REMOVE = "remove",
  FINALIZE = "finalize",
  TRANSFER_START = "transfer-start",
  TRANSFER_FINISH = "transfer-finish",
  REVOKE = "revoke",
}

export enum LinkStatus {
  RESUME = "RESUME",
  LINKED = "LINKED",
  AUTHENTICATOR_PRESENT = "AUTHENTICATOR_PRESENT",
  NEEDS_PHONE = "NEEDS_PHONE",
  NEEDS_EMAIL = "NEEDS_EMAIL",
}

export enum ConfirmationKind {
  TRADE = "TRADE",
  MARKET = "MARKET",
  PHONE = "PHONE",
  RECOVERY = "RECOVERY",
  API_KEY = "API_KEY",
  FEATURE = "FEATURE",
  FAMILY = "FAMILY",
  TEST = "TEST",
  OTHER = "OTHER",
}

export enum LinkConfirmKind {
  SMS = "SMS",
  EMAIL = "EMAIL",
  UNKNOWN = "UNKNOWN",
}

export type ConfirmationActionType = `confirmation:${string}:${boolean}`;
export type GuardBusyActionType = GuardAction | ConfirmationActionType;

export enum SteamGuardField {
  ACCOUNT_NAME = "account-name",
  PASSWORD = "password",
  LOGIN_CODE = "login-code",
  LOGIN = "login",
  IMPORT_FILE = "import-file",
  IMPORT_PASSWORD = "import-password",
  IMPORT = "import",
  LINK_CODE = "link-code",
  TRANSFER = "transfer",
  REVOCATION = "revocation",
  LINK = "link",
  SECURITY = "security",
  CONFIRMATIONS = "confirmations",
  GENERAL = "general",
}
