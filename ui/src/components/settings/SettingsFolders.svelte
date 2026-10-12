<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { FolderAction } from "./SettingsFolders.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import { createFolder, deleteFolder } from "../../lib/actions/actions.svelte";
  import { folders } from "../../lib/state/state.svelte";
  import { openFolderEditor } from "../../lib/state/dialogs.svelte";
  import { Folder, Plus, SquarePen, Trash2 } from "@lucide/svelte";
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import type { FolderType } from "../../lib/types";

  let name = $state("");
  let creating = $state(false);
  let pending = $state<Partial<Record<string, FolderAction>>>({});
  const list = $derived(folders());

  const create = async () => {
    const folderName = name.trim();
    if (!folderName || creating) {
      return;
    }
    creating = true;
    try {
      if ((await createFolder(folderName)) && name.trim() === folderName) name = "";
    } finally {
      creating = false;
    }
  };

  const remove = async (folder: FolderType): Promise<void> => {
    if (pending[folder.id]) return;
    pending[folder.id] = FolderAction.DELETE;
    try {
      await deleteFolder(folder.id);
    } finally {
      delete pending[folder.id];
    }
  };
</script>

<div class="settings-cards">
  <section class="settings-card wide">
    <h3 class="settings-card-title"><Folder size={15} /> {t.settings_folders_title()}</h3>
    <p class="settings-note">{t.folder_settings_description()}</p>
    <div class="settings-list">
      {#if list.length === 0}
        <p class="settings-note">{t.folder_empty()}</p>
      {:else}
        {#each list as folder (folder.id)}
          {@const Glyph = folderIcon(folder.icon)}
          <div class="settings-row">
            <span class="folder-row-icon" style={tintStyle(folder.color)} aria-hidden="true">
              <Glyph size={16} />
            </span>
            <span class="settings-label">{folder.name}</span>
            <span class="tag">{t.accounts_count({ count: folder.account_count })}</span>
            <button
              class="button"
              type="button"
              disabled={Boolean(pending[folder.id])}
              onclick={() => openFolderEditor(folder.id)}
              ><SquarePen size={15} /> {t.folder_edit_action()}</button
            >
            <ActionButton
              icon={Trash2}
              label={t.common_delete()}
              busyLabel={t.common_deleting()}
              busy={pending[folder.id] === FolderAction.DELETE}
              disabled={Boolean(pending[folder.id])}
              className="button danger"
              onclick={() => remove(folder)}
            />
          </div>
        {/each}
      {/if}
    </div>
    <div class="settings-add">
      <input
        class="input"
        type="text"
        placeholder={t.folder_create_placeholder()}
        aria-label={t.folder_create_placeholder()}
        spellcheck="false"
        bind:value={name}
        onkeydown={(event) => event.key === "Enter" && (event.preventDefault(), void create())}
      />
      <ActionButton
        icon={Plus}
        label={t.common_add()}
        busyLabel={t.common_adding()}
        busy={creating}
        disabled={!name.trim()}
        className="button primary"
        onclick={create}
      />
    </div>
  </section>
</div>
