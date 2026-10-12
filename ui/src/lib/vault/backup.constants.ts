export enum BackupMode {
  EXPORT = "export",
  IMPORT = "import",
}

export enum BackupProtection {
  PASSWORD = "password",
  NONE = "none",
}

export enum BackupField {
  FILE = "file",
  PASSWORD = "password",
  REPEAT = "repeat",
  GENERAL = "general",
}

export enum PickFileKind {
  MAFILE = "mafile",
  BACKUP = "backup",
}

export const BACKUP_EXTENSION = "steamguard";
