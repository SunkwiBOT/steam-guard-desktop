import type { Language } from "../i18n/i18n.constants";
import { locale, t } from "../i18n/i18n.svelte";
import { checkForUpdateIfDue } from "../actions/actions.svelte";
import { invoke } from "../api/tauri";
import { ToastKind } from "./state.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "./state.constants";
import { app, defaultSettingsState, run } from "./state.svelte";
import { toast } from "./toasts";

export type SettingsDraftType = {
  language: Language;
  searchScope: SearchScope;
  autostart: boolean;
  closeToTray: boolean;
  showAccountNames: boolean;
  blurCodes: boolean;
  showSteamIds: boolean;
  updateInterval: UpdateInterval;
  vaultLockInterval: VaultLockInterval;
};

let autostartAvailable = $state<boolean>(true);
let committedAutostart = $state<boolean>(false);
let autostartLoaded = $state<boolean>(false);

// ! `autostart_status` fails (never returns null) only when the platform's autostart backend
// is unavailable; that rejection is the only signal that the toggle should be disabled.
const loadAutostart = async (): Promise<void> => {
  try {
    committedAutostart = await invoke("autostart_status");
    autostartAvailable = true;
  } catch (e: unknown) {
    autostartAvailable = false;
  } finally {
    autostartLoaded = true;
  }
};
void loadAutostart();

export const settingsAutostartAvailable = (): boolean => autostartAvailable;

const settings = $derived(app.snapshot?.settings ?? defaultSettingsState.value);

const baseline = $derived<SettingsDraftType | null>(
  settings && autostartLoaded
    ? {
        language: locale.preference,
        searchScope: settings.search_scope,
        autostart: committedAutostart,
        closeToTray: settings.close_to_tray,
        showAccountNames: settings.show_account_names,
        blurCodes: settings.blur_codes,
        showSteamIds: settings.show_steam_ids,
        updateInterval: settings.update_interval,
        vaultLockInterval: settings.vault_lock_interval,
      }
    : null,
);

// ! Only the fields the user actually touched: a baseline value that arrives after the draft
// mounts (autostart resolves async) can never be mistaken for an edit.
const overrides = $state<Partial<SettingsDraftType>>({});
let saving = $state(false);

const current = $derived<SettingsDraftType | null>(baseline ? { ...baseline, ...overrides } : null);

export const settingsDraft = {
  get value(): SettingsDraftType | null {
    return current;
  },
  get dirty(): boolean {
    return Object.keys(overrides).length > 0;
  },
  get saving(): boolean {
    return saving;
  },
  set<K extends keyof SettingsDraftType>(key: K, value: SettingsDraftType[K]): void {
    if (!baseline) return;
    if (baseline[key] === value) delete overrides[key];
    else overrides[key] = value;
  },
  cancel(): void {
    if (saving) return;
    for (const key of Object.keys(overrides) as (keyof SettingsDraftType)[]) {
      delete overrides[key];
    }
  },
  async save(): Promise<void> {
    if (saving || !baseline || !current) return;
    const snapshot = current;
    const changed = new Set(Object.keys(overrides) as (keyof SettingsDraftType)[]);
    saving = true;
    try {
      const tasks: Promise<unknown>[] = [];
      if (changed.has("language")) {
        tasks.push(
          run("set_language", { language: snapshot.language }).then((result) => {
            if (result) delete overrides.language;
          }),
        );
      }
      if (changed.has("searchScope")) {
        tasks.push(
          run("set_search_scope", { scope: snapshot.searchScope }).then((result) => {
            if (result) delete overrides.searchScope;
          }),
        );
      }
      if (changed.has("closeToTray")) {
        tasks.push(
          run("set_close_to_tray", { closeToTray: snapshot.closeToTray }).then((result) => {
            if (result) delete overrides.closeToTray;
          }),
        );
      }
      if (changed.has("showAccountNames")) {
        tasks.push(
          run("set_show_account_names", { visible: snapshot.showAccountNames }).then((result) => {
            if (result) delete overrides.showAccountNames;
          }),
        );
      }
      if (changed.has("blurCodes")) {
        tasks.push(
          run("set_blur_codes", { enabled: snapshot.blurCodes }).then((result) => {
            if (result) delete overrides.blurCodes;
          }),
        );
      }
      if (changed.has("showSteamIds")) {
        tasks.push(
          run("set_steam_ids_visible", { visible: snapshot.showSteamIds }).then((result) => {
            if (result) delete overrides.showSteamIds;
          }),
        );
      }
      if (changed.has("updateInterval")) {
        tasks.push(
          run("set_update_interval", { interval: snapshot.updateInterval }).then((result) => {
            if (result) {
              delete overrides.updateInterval;
              void checkForUpdateIfDue();
            }
          }),
        );
      }
      if (changed.has("vaultLockInterval")) {
        tasks.push(
          run("set_vault_lock_interval", { interval: snapshot.vaultLockInterval }).then(
            (result) => {
              if (result) delete overrides.vaultLockInterval;
            },
          ),
        );
      }
      if (changed.has("autostart")) {
        tasks.push(
          invoke("set_autostart", { enabled: snapshot.autostart })
            .then((result) => {
              committedAutostart = result;
              delete overrides.autostart;
              toast(
                ToastKind.SUCCESS,
                result ? t.settings_autostart_enabled() : t.settings_autostart_disabled(),
              );
            })
            .catch(() => {
              toast(ToastKind.ERROR, t.action_failed_title());
            }),
        );
      }
      await Promise.all(tasks);
    } finally {
      saving = false;
    }
  },
};
