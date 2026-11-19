<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import FolderContextMenu from "./FolderContextMenu.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderCardPropsType = { folder: FolderDto };

  let { folder }: FolderCardPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
</script>

<FolderContextMenu {folder}>
  {#snippet children({ props })}
    <button
      {...props}
      type="button"
      class="card folder-card group relative flex h-full min-h-[168px] flex-col items-center justify-start gap-1.5 overflow-hidden rounded-md border border-[var(--border)] bg-[var(--base-alt)] px-2.5 pt-2.5 pb-2 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:border-[var(--border-strong)] hover:bg-[var(--surface-hover)] focus-visible:shadow-[inset_0_0_0_2px_var(--accent)] focus-visible:outline-none [&_img]:pointer-events-none [&_img]:[-webkit-user-drag:none] [&_img]:select-none [&_svg]:pointer-events-none [&_svg]:select-none"
      title={t.folder_card_tooltip()}
      onclick={() => (app.folderId = folder.id)}
    >
      <span
        class="mt-1 flex size-[62px] shrink-0 items-center justify-center rounded-full bg-[var(--avatar)] text-[var(--accent)]"
        style={tintStyle(folder.color)}
        aria-hidden="true"
      >
        <Glyph size={32} />
      </span>
      <span
        class="w-full text-center font-semibold whitespace-normal text-[var(--text-strong)] [overflow-wrap:anywhere]"
        >{folder.name}</span
      >
      <div
        class="mt-auto flex min-h-5 max-w-full items-center justify-center gap-1 text-[11px] text-[var(--muted)]"
      >
        <span>{t.accounts_count({ count: folder.account_count })}</span>
      </div>
    </button>
  {/snippet}
</FolderContextMenu>
