<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { openFolderMenu } from "../../lib/state/menus.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import type { FolderType } from "../../lib/types";

  type FolderCardPropsType = { folder: FolderType };

  let { folder }: FolderCardPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
</script>

<button
  type="button"
  class="card folder-card"
  title={t.folder_card_tooltip()}
  onclick={() => (app.folderId = folder.id)}
  oncontextmenu={(event) => openFolderMenu(event, folder.id)}
>
  <span class="folder-icon" style={tintStyle(folder.color)} aria-hidden="true">
    <Glyph size={32} />
  </span>
  <span class="card-name">{folder.name}</span>
  <div class="card-meta">
    <span>{t.accounts_count({ count: folder.account_count })}</span>
  </div>
</button>
