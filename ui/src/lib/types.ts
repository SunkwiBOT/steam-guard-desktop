import type { Language, ResolvedLanguageType } from "./i18n/i18n.constants";
import type {
  ConfirmationKind,
  GuardKind,
  LinkConfirmKind,
  LinkStatus,
} from "./steam-guard/steam-guard.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "./state/state.constants";

export type SettingsType = {
  language: Language;
  resolved_language: ResolvedLanguageType;
  close_to_tray: boolean;
  show_account_names: boolean;
  show_steam_ids: boolean;
  blur_codes: boolean;
  update_interval: UpdateInterval;
  search_scope: SearchScope;
  vault_lock_interval: VaultLockInterval;
  last_update_check_at: number | null;
};

export type LogSnapshotType = {
  path: string;
  content: string;
  truncated: boolean;
  previous_available: boolean;
};

export type AccountType = {
  steam_id: string;
  account_name: string;
  persona_name: string;
  display_name: string;
  initial: string;
  avatar: string | null;
  has_tokens: boolean;
  pending: boolean;
  profile_fetched_at: number | null;
  color: string | null;
  folder_id: string | null;
};

export type ToastAccountType = Pick<
  AccountType,
  "avatar" | "initial" | "display_name" | "persona_name"
>;

export type FolderType = {
  id: string;
  name: string;
  account_count: number;
  /** ! Always false in this app: kept so the contract matches the switcher. */
  system: boolean;
  color: string | null;
  icon: string | null;
};

export type ToastFolderType = Pick<FolderType, "name" | "color" | "icon">;

export type VaultStatusType = {
  path: string;
  lock_in_seconds: number | null;
  encrypted: boolean;
  locked: boolean;
  indexed: boolean;
  count: number;
  error: string | null;
};

export type StoredAuthenticatorType = {
  steam_id: string;
  account_name: string;
  serial_number: string;
  device_id: string;
  has_tokens: boolean;
};

export type CodeInfoType = {
  valid_until: number;
  code: string;
  seconds_remaining: number;
  period: number;
};

export type ConfirmationItemType = {
  id: string;
  nonce: string;
  kind: ConfirmationKind;
  headline: string;
  summary: string;
  creator_id: string;
  creation_time: number;
  icon: string | null;
  multi: boolean;
};

export type GuardMethodType = {
  kind: GuardKind;
  message: string;
};

export type LoginBeginType = {
  session_id: string;
  methods: GuardMethodType[];
  qr_svg: string | null;
  challenge_url: string | null;
};

export type LoginResultType = {
  session_id: string;
  steam_id: string;
  account_name: string;
  stored: boolean;
};

export type LinkStartType = {
  session_id: string;
  status: LinkStatus;
  steam_id: string;
  account_name: string;
  revocation_code: string | null;
  phone_hint: string | null;
  confirm_kind: LinkConfirmKind | null;
};

export type SnapshotType = {
  revision: number;
  accounts: AccountType[];
  vault: VaultStatusType;
  settings: SettingsType;
  folders: FolderType[];
  masked: boolean;
  steam_ids_visible: boolean;
};

export type UpdateInfoType = {
  version: string;
  current_version: string;
  notes: string | null;
};

export type UpdateProgressType = {
  downloaded: number;
  total: number | null;
};

export type AppInfoType = {
  name: string;
  version: string;
  repository: string;
};

export type AvatarSourceType = {
  avatar: string | null;
  initial: string;
  persona_name?: string;
};

export type BackupImportType = {
  imported: number;
  skipped: number;
  snapshot: SnapshotType;
};
