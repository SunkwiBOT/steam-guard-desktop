<script lang="ts">
  import { FOLDER_ICONS } from "../../lib/folders/folder-icons";
  import { folderIconId } from "../../lib/folders/folder-style";
  import { app } from "../../lib/state/state.svelte";
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";

  type FolderIconPickerPropsType = {
    value: string | null | undefined;
    color?: string | null;
    className?: string;
    disabled?: boolean;
    asMenuItems?: boolean;
    onselect: (icon: string) => unknown;
  };

  let {
    value,
    color = null,
    className = "icon-grid",
    disabled = false,
    asMenuItems = false,
    onselect,
  }: FolderIconPickerPropsType = $props();
  const current = $derived(folderIconId(value, app.info.folder_icons));
</script>

<div class={className} style={`color:${color || "var(--muted)"}`}>
  {#each FOLDER_ICONS as entry (entry.id)}
    {#if asMenuItems}
      <ContextMenuPrimitive.Item
        closeOnSelect={false}
        {disabled}
        onSelect={() => onselect(entry.id)}
      >
        {#snippet child({ props })}
          <button
            {...props}
            class="icon-choice"
            class:active={current === entry.id}
            type="button"
            title={entry.label()}
            aria-label={entry.label()}
            aria-pressed={current === entry.id}
            {disabled}
          >
            <entry.icon size={18} />
          </button>
        {/snippet}
      </ContextMenuPrimitive.Item>
    {:else}
      <button
        class="icon-choice"
        class:active={current === entry.id}
        type="button"
        title={entry.label()}
        aria-label={entry.label()}
        aria-pressed={current === entry.id}
        {disabled}
        onclick={() => onselect(entry.id)}
      >
        <entry.icon size={18} />
      </button>
    {/if}
  {/each}
</div>
