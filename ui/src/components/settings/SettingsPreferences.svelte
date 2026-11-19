<script lang="ts">
  import { Language, LANGUAGE_LABELS, RESOLVED_LANGUAGES } from "../../lib/i18n/i18n.constants";
  import { SearchScope, UpdateInterval } from "../../lib/state/state.constants";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { settingsAutostartAvailable, settingsDraft } from "../../lib/state/settings-draft.svelte";
  import SettingToggle from "./SettingToggle.svelte";
  import SelectSetting from "../ui/SelectSetting.svelte";

  import { Download, Eye, Languages, Power, Search } from "@lucide/svelte";

  const draft = $derived(settingsDraft.value);

  const languageOptions = $derived([
    { value: Language.AUTO, label: t.language_system() },
    ...RESOLVED_LANGUAGES.map((value) => ({ value, label: LANGUAGE_LABELS[value] })),
  ]);
  const languageLabel = $derived(
    languageOptions.find((option) => option.value === draft?.language)?.label ??
      t.language_system(),
  );

  const searchScopes = $derived([
    { value: SearchScope.BOTH, label: t.settings_search_both() },
    { value: SearchScope.DISPLAY_NAME, label: t.settings_search_display_name() },
    { value: SearchScope.IDENTIFIER, label: t.settings_search_identifier() },
  ]);
  const searchScopeLabel = $derived(
    searchScopes.find((option) => option.value === draft?.searchScope)?.label ??
      t.settings_search_both(),
  );

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
    intervals.find((option) => option.value === draft?.updateInterval)?.label ??
      t.updates_check_6_hours(),
  );
</script>

{#if draft}
  <div class="settings-cards">
    <section class="settings-card">
      <h3 class="settings-card-title"><Languages size={15} /> {t.language_label()}</h3>
      <p class="settings-note">{t.language_description()}</p>
      <SelectSetting
        value={draft.language}
        options={languageOptions}
        label={languageLabel}
        ariaLabel={t.language_label()}
        onselect={(value) => {
          const language = Object.values(Language).find((choice) => choice === value);
          if (language) settingsDraft.set("language", language);
        }}
      />
    </section>
    <section class="settings-card">
      <h3 class="settings-card-title"><Search size={15} /> {t.settings_search_title()}</h3>
      <p class="settings-note">{t.settings_search_description()}</p>
      <SelectSetting
        value={draft.searchScope}
        options={searchScopes}
        label={searchScopeLabel}
        ariaLabel={t.settings_search_label()}
        onselect={(value) => {
          const scope = searchScopes.find((choice) => choice.value === value)?.value;
          if (scope) settingsDraft.set("searchScope", scope);
        }}
      />
    </section>
    <section class="settings-card">
      <h3 class="settings-card-title"><Power size={15} /> {t.settings_startup_title()}</h3>
      <SettingToggle
        label={t.settings_autostart_label()}
        checked={draft.autostart}
        disabled={!settingsAutostartAvailable()}
        onchange={async (enabled) => settingsDraft.set("autostart", enabled)}
      />
      <SettingToggle
        label={t.settings_close_quit_label()}
        checked={draft.closeToTray === false}
        onchange={async (enabled) => settingsDraft.set("closeToTray", !enabled)}
      />
      <p class="settings-note">
        {draft.closeToTray === false
          ? t.settings_close_quit_description()
          : t.settings_close_tray_description()}
      </p>
    </section>
    <section class="settings-card">
      <h3 class="settings-card-title"><Eye size={15} /> {t.settings_privacy_title()}</h3>
      <SettingToggle
        label={t.settings_privacy_show_login_names()}
        checked={draft.showAccountNames !== false}
        onchange={async (enabled) => settingsDraft.set("showAccountNames", enabled)}
      />
      <p class="settings-note">{t.settings_privacy_description()}</p>
      <SettingToggle
        label={t.settings_privacy_blur_codes()}
        checked={draft.blurCodes === true}
        onchange={async (enabled) => settingsDraft.set("blurCodes", enabled)}
      />
      <p class="settings-note">{t.settings_privacy_blur_description()}</p>
      <SettingToggle
        label={t.settings_privacy_hide_ids()}
        checked={draft.showSteamIds !== true}
        onchange={async (enabled) => settingsDraft.set("showSteamIds", !enabled)}
      />
      <p class="settings-note">{t.settings_privacy_ids_description()}</p>
    </section>
    <section class="settings-card">
      <h3 class="settings-card-title"><Download size={15} /> {t.updates_schedule_label()}</h3>
      <SelectSetting
        value={draft.updateInterval}
        options={intervals}
        label={intervalLabel}
        ariaLabel={t.updates_schedule_label()}
        onselect={(value) => {
          const interval = intervals.find((choice) => choice.value === value)?.value;
          if (interval) settingsDraft.set("updateInterval", interval);
        }}
      />
      <p class="settings-note">
        {draft.updateInterval === UpdateInterval.NEVER
          ? t.updates_checks_disabled()
          : draft.updateInterval === UpdateInterval.STARTUP
            ? t.updates_checks_startup_description()
            : t.updates_checks_description()}
      </p>
    </section>
  </div>
{/if}
