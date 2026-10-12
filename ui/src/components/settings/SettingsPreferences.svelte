<script lang="ts">
  import { Language, LANGUAGE_LABELS, RESOLVED_LANGUAGES } from "../../lib/i18n/i18n.constants";
  import { SearchScope, UpdateInterval } from "../../lib/state/state.constants";
  import { locale, t } from "../../lib/i18n/i18n.svelte";
  import { dropdown, openDropdown } from "../../lib/state/menus.svelte";
  import { checkForUpdateIfDue } from "../../lib/actions/actions.svelte";

  import { app, EMPTY_SETTINGS, run, showToast } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import type { SettingsType } from "../../lib/types";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import SettingToggle from "./SettingToggle.svelte";

  import { ChevronDown, Download, Eye, Languages, Power, Search } from "@lucide/svelte";

  const settings = $derived<SettingsType>(app.snapshot?.settings ?? EMPTY_SETTINGS);

  let autostartEnabled = $state(false);
  let autostartAvailable = $state(true);
  let languageBusy = $state(false);
  let languageButton = $state<HTMLButtonElement | null>(null);
  const languageOptions = $derived([
    { value: Language.AUTO, label: t.language_system() },
    ...RESOLVED_LANGUAGES.map((value) => ({ value, label: LANGUAGE_LABELS[value] })),
  ]);
  const languageLabel = $derived(
    languageOptions.find((option) => option.value === locale.preference)?.label ??
      t.language_system(),
  );

  const changeLanguage = async (language: Language): Promise<void> => {
    if (languageBusy || language === locale.preference) return;
    languageBusy = true;
    try {
      await run("set_language", { language });
    } finally {
      languageBusy = false;
    }
  };

  const pickLanguage = (): void => {
    if (!languageButton || languageBusy) return;
    openDropdown({
      anchor: languageButton,
      value: locale.preference,
      options: languageOptions,
      onSelect: (value) => {
        const language = Object.values(Language).find((choice) => choice === value);
        if (language) void changeLanguage(language);
      },
    });
  };

  let searchBusy = $state(false);
  let searchButton = $state<HTMLButtonElement | null>(null);
  const searchScopes = $derived([
    { value: SearchScope.BOTH, label: t.settings_search_both() },
    { value: SearchScope.DISPLAY_NAME, label: t.settings_search_display_name() },
    { value: SearchScope.IDENTIFIER, label: t.settings_search_identifier() },
  ]);
  const searchScopeLabel = $derived(
    searchScopes.find((option) => option.value === settings.search_scope)?.label ??
      t.settings_search_both(),
  );

  const changeSearchScope = async (scope: SearchScope): Promise<void> => {
    if (searchBusy || scope === settings.search_scope) return;
    searchBusy = true;
    try {
      await run("set_search_scope", { scope });
    } finally {
      searchBusy = false;
    }
  };

  const pickSearchScope = (): void => {
    if (!searchButton || searchBusy) return;
    openDropdown({
      anchor: searchButton,
      value: settings.search_scope,
      options: searchScopes,
      onSelect: (value) => {
        const scope = Object.values(SearchScope).find((choice) => choice === value);
        if (scope) void changeSearchScope(scope);
      },
    });
  };

  const intervals = $derived([
    { value: UpdateInterval.NEVER, label: t.updates_check_never() },
    { value: UpdateInterval.STARTUP, label: t.updates_check_startup() },
    { value: UpdateInterval.FIFTEEN_MINUTES, label: t.updates_check_15_minutes() },
    { value: UpdateInterval.THIRTY_MINUTES, label: t.updates_check_30_minutes() },
    { value: UpdateInterval.ONE_HOUR, label: t.updates_check_hourly() },
    { value: UpdateInterval.SIX_HOURS, label: t.updates_check_6_hours() },
    { value: UpdateInterval.ONE_DAY, label: t.updates_check_daily() },
    { value: UpdateInterval.ONE_WEEK, label: t.updates_check_weekly() },
  ]);
  const intervalLabel = $derived(
    intervals.find((option) => option.value === settings.update_interval)?.label ??
      t.updates_check_6_hours(),
  );
  let intervalBusy = $state(false);
  let intervalButton = $state<HTMLButtonElement | null>(null);

  const changeInterval = async (interval: UpdateInterval): Promise<void> => {
    if (intervalBusy || interval === settings.update_interval) return;
    intervalBusy = true;
    try {
      if (await run("set_update_interval", { interval })) void checkForUpdateIfDue();
    } finally {
      intervalBusy = false;
    }
  };

  const pickInterval = (): void => {
    if (intervalBusy || !intervalButton) return;
    openDropdown({
      anchor: intervalButton,
      value: settings.update_interval,
      options: intervals,
      onSelect: (value) => {
        const interval = Object.values(UpdateInterval).find((choice) => choice === value);
        if (interval) void changeInterval(interval);
      },
    });
  };

  $effect(() => {
    invoke("autostart_status")
      .then((enabled: boolean) => {
        autostartAvailable = enabled !== null;
        autostartEnabled = enabled === true;
      })
      .catch(() => {
        autostartAvailable = false;
      });
  });

  const toggleAutostart = async (enabled: boolean): Promise<void> => {
    const result = await run("set_autostart", { enabled });
    if (result === null) {
      autostartEnabled = !enabled;
    } else {
      autostartEnabled = enabled;
      showToast(enabled ? t.settings_autostart_enabled() : t.settings_autostart_disabled(), true);
    }
  };
</script>

<div class="settings-cards">
  <section class="settings-card">
    <h3 class="settings-card-title"><Languages size={15} /> {t.language_label()}</h3>
    <p class="settings-note">{t.language_description()}</p>
    <button
      class="select-button"
      type="button"
      aria-label={t.language_label()}
      aria-haspopup="listbox"
      aria-expanded={dropdown.open && dropdown.anchor === languageButton}
      bind:this={languageButton}
      onclick={pickLanguage}
      disabled={languageBusy}
      aria-busy={languageBusy}
    >
      <span>{languageBusy ? t.common_saving() : languageLabel}</span>
      <span class="picker-caret">
        <AsyncIcon icon={ChevronDown} busy={languageBusy} size={14} />
      </span>
    </button>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Search size={15} /> {t.settings_search_title()}</h3>
    <p class="settings-note">{t.settings_search_description()}</p>
    <button
      class="select-button"
      type="button"
      aria-label={t.settings_search_label()}
      aria-haspopup="listbox"
      aria-expanded={dropdown.open && dropdown.anchor === searchButton}
      bind:this={searchButton}
      onclick={pickSearchScope}
      disabled={searchBusy}
      aria-busy={searchBusy}
    >
      <span>{searchBusy ? t.common_saving() : searchScopeLabel}</span>
      <span class="picker-caret">
        <AsyncIcon icon={ChevronDown} busy={searchBusy} size={14} />
      </span>
    </button>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Power size={15} /> {t.settings_startup_title()}</h3>
    <SettingToggle
      label={t.settings_autostart_label()}
      checked={autostartEnabled}
      disabled={!autostartAvailable}
      onchange={toggleAutostart}
    />
    <SettingToggle
      label={t.settings_close_quit_label()}
      checked={settings.close_to_tray === false}
      onchange={(enabled) => run("set_close_to_tray", { closeToTray: !enabled })}
    />
    <p class="settings-note">
      {settings.close_to_tray === false
        ? t.settings_close_quit_description()
        : t.settings_close_tray_description()}
    </p>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Eye size={15} /> {t.settings_privacy_title()}</h3>
    <SettingToggle
      label={t.settings_privacy_show_login_names()}
      checked={settings.show_account_names !== false}
      onchange={(enabled) => run("set_show_account_names", { visible: enabled })}
    />
    <p class="settings-note">{t.settings_privacy_description()}</p>
    <SettingToggle
      label={t.settings_privacy_blur_codes()}
      checked={settings.blur_codes === true}
      onchange={(enabled) => run("set_blur_codes", { enabled })}
    />
    <p class="settings-note">{t.settings_privacy_blur_description()}</p>
  </section>
  <section class="settings-card">
    <h3 class="settings-card-title"><Download size={15} /> {t.updates_schedule_label()}</h3>
    <button
      class="select-button"
      type="button"
      aria-label={t.updates_schedule_label()}
      aria-haspopup="listbox"
      aria-expanded={dropdown.open && dropdown.anchor === intervalButton}
      aria-busy={intervalBusy}
      bind:this={intervalButton}
      disabled={intervalBusy}
      onclick={pickInterval}
    >
      <span>{intervalBusy ? t.common_saving() : intervalLabel}</span>
      <span class="picker-caret"
        ><AsyncIcon icon={ChevronDown} busy={intervalBusy} size={14} /></span
      >
    </button>
    <p class="settings-note">
      {settings.update_interval === UpdateInterval.NEVER
        ? t.updates_checks_disabled()
        : settings.update_interval === UpdateInterval.STARTUP
          ? t.updates_checks_startup_description()
          : t.updates_checks_description()}
    </p>
  </section>
</div>
