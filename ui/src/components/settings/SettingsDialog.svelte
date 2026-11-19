<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { Tabs, TabsList, TabsTrigger, TabsContent } from "$lib/components/ui/tabs/index.js";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { SettingsSection } from "../../lib/state/state.constants";
  import { app } from "../../lib/state/state.svelte";
  import { settingsDraft } from "../../lib/state/settings-draft.svelte";
  import { Download, FileText, Folder, Lock, Settings, SlidersHorizontal, X } from "@lucide/svelte";
  import SettingsFolders from "./SettingsFolders.svelte";
  import SettingsPreferences from "./SettingsPreferences.svelte";
  import SettingsUpdates from "./SettingsUpdates.svelte";
  import SettingsLogs from "./SettingsLogs.svelte";
  import UnsavedBar from "../ui/UnsavedBar.svelte";

  import VaultControls from "../vault/VaultControls.svelte";
  import VaultBackups from "../vault/VaultBackups.svelte";

  const sections = $derived([
    {
      id: SettingsSection.PREFERENCES,
      label: t.settings_preferences_title(),
      icon: SlidersHorizontal,
    },
    { id: SettingsSection.FOLDERS, label: t.settings_folders_title(), icon: Folder },
    { id: SettingsSection.VAULT, label: t.vault_section_title(), icon: Lock },
    { id: SettingsSection.LOGS, label: t.settings_logs_title(), icon: FileText },
    { id: SettingsSection.UPDATES, label: t.settings_updates_title(), icon: Download },
  ]);
  const uid = $props.id();

  // ! A dialog closed while a save request is in flight must not lose its target; a dirty,
  // idle draft refuses the close so Cancel/Save stays the only way out of an unsaved edit.
  const closeSettings = (): void => {
    if (settingsDraft.saving || settingsDraft.dirty) return;
    app.settingsOpen = false;
  };
</script>

<AppDialog open={app.settingsOpen} onclose={closeSettings}>
  <DialogContent
    showCloseButton={false}
    class="sm:max-w-3xl max-h-[86vh] flex flex-col overflow-hidden"
    aria-labelledby={`${uid}-title`}
  >
    <DialogHeader
      id={`${uid}-title`}
      title={t.settings_title()}
      icon={Settings}
      onclose={closeSettings}
      closeDisabled={settingsDraft.saving || settingsDraft.dirty}
    />

    <div class="settings-layout">
      <Tabs
        value={app.settingsSection}
        onValueChange={(value) => (app.settingsSection = value as SettingsSection)}
        orientation="vertical"
        class="contents"
      >
        <TabsList aria-label={t.settings_navigation_label()}>
          {#each sections as section (section.id)}
            <TabsTrigger value={section.id}>
              <section.icon size={15} />
              {section.label}
            </TabsTrigger>
          {/each}
        </TabsList>

        <div class="settings-body">
          <TabsContent value={SettingsSection.PREFERENCES}>
            {#if app.settingsSection === SettingsSection.PREFERENCES}<SettingsPreferences />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.FOLDERS}>
            {#if app.settingsSection === SettingsSection.FOLDERS}<SettingsFolders />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.VAULT}>
            {#if app.settingsSection === SettingsSection.VAULT}
              <section class="settings-card">
                <h3 class="settings-card-title"><Lock size={15} /> {t.vault_title()}</h3>
                <VaultControls showSettings />
              </section>
              <VaultBackups />
            {/if}
          </TabsContent>
          <TabsContent value={SettingsSection.LOGS}>
            {#if app.settingsSection === SettingsSection.LOGS}<SettingsLogs />{/if}
          </TabsContent>
          <TabsContent value={SettingsSection.UPDATES}>
            {#if app.settingsSection === SettingsSection.UPDATES}<SettingsUpdates />{/if}
          </TabsContent>
        </div>
      </Tabs>
    </div>

    {#if settingsDraft.dirty}
      <UnsavedBar
        saving={settingsDraft.saving}
        oncancel={() => settingsDraft.cancel()}
        onsave={() => void settingsDraft.save()}
      />
    {/if}

    <footer class="dialog-foot">
      <button
        class="button"
        type="button"
        disabled={settingsDraft.saving || settingsDraft.dirty}
        onclick={closeSettings}><X size={15} /> {t.common_close()}</button
      >
    </footer>
  </DialogContent>
</AppDialog>
