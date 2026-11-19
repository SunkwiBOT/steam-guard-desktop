import { ErrorCode } from "../bindings/ErrorCode";

// ! These six codes never cross IPC: they are raised by the page's own form
// validation before a command runs, so they have no `guard_core::error_code::ErrorCode` variant.
const CLIENT_ERROR_CODES = [
  "ACCOUNT_NAME_REQUIRED",
  "BACKUP_FILE_REQUIRED",
  "GUARD_CODE_REQUIRED",
  "PASSWORD_MISMATCH",
  "REVOCATION_CODE_REQUIRED",
  "STEAM_PASSWORD_REQUIRED",
] as const;

type ClientErrorCode = (typeof CLIENT_ERROR_CODES)[number];

export type UiErrorCode = ErrorCode | ClientErrorCode;

export const UiErrorCode = {
  ...ErrorCode,
  ACCOUNT_NAME_REQUIRED: "ACCOUNT_NAME_REQUIRED",
  BACKUP_FILE_REQUIRED: "BACKUP_FILE_REQUIRED",
  GUARD_CODE_REQUIRED: "GUARD_CODE_REQUIRED",
  PASSWORD_MISMATCH: "PASSWORD_MISMATCH",
  REVOCATION_CODE_REQUIRED: "REVOCATION_CODE_REQUIRED",
  STEAM_PASSWORD_REQUIRED: "STEAM_PASSWORD_REQUIRED",
} as const satisfies Record<string, UiErrorCode>;

const codes = new Set<string>([...Object.values(ErrorCode), ...CLIENT_ERROR_CODES]);

export const isUiErrorCode = (value: string): value is UiErrorCode => codes.has(value);

export const resolveErrorCode = (value: unknown): UiErrorCode => {
  const code = value instanceof Error ? value.message : value;
  return typeof code === "string" && isUiErrorCode(code) ? code : UiErrorCode.UNKNOWN_ERROR;
};

// ! A vault written by a newer build must never be offered for overwrite or restore.
export const isVaultRequiresNewerVersion = (value: unknown): boolean =>
  resolveErrorCode(value) === UiErrorCode.VAULT_REQUIRES_NEWER_VERSION;
