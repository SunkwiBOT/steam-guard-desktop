<script lang="ts">
  import { FOLDER_ICONS } from "../../lib/folders/folder-icons";
  import { folderIconId } from "../../lib/folders/folder-style";

  type FolderIconPickerPropsType = {
    value: string | null | undefined;
    color?: string | null;
    className?: string;
    disabled?: boolean;
    onselect: (icon: string) => unknown;
  };

  let {
    value,
    color = null,
    className = "icon-grid",
    disabled = false,
    onselect,
  }: FolderIconPickerPropsType = $props();
  const current = $derived(folderIconId(value));
</script>

<div class={className} style={`color:${color || "var(--muted)"}`}>
  {#each FOLDER_ICONS as entry (entry.id)}
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
  {/each}
</div>
