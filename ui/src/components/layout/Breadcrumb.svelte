<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { folderIcon } from "../../lib/folders/folder-icons";
  import { app, currentFolder } from "../../lib/state/state.svelte";

  const folder = $derived(currentFolder());
  const Glyph = $derived(folderIcon(folder?.icon));
</script>

<nav class="breadcrumb" aria-label={t.folder_path()}>
  <button
    type="button"
    class="crumb"
    class:current={app.folderId === null}
    onclick={() => (app.folderId = null)}
  >
    {t.accounts_all()}
  </button>
  {#if folder}
    <span class="crumb-sep">/</span>
    <button
      type="button"
      class="crumb current"
      style={folder.color ? `color:${folder.color}` : ""}
      onclick={() => (app.folderId = folder.id)}
    >
      <Glyph size={13} />
      {folder.name}
    </button>
  {/if}
</nav>
