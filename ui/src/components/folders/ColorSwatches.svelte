<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { X } from "@lucide/svelte";
  import { CARD_COLORS } from "../../lib/folders/folder-style";
  import { ContextMenu as ContextMenuPrimitive } from "bits-ui";

  type ColorSwatchesPropsType = {
    value: string | null | undefined;
    className?: string;
    disabled?: boolean;
    asMenuItems?: boolean;
    onselect: (color: string) => unknown;
  };

  let {
    value,
    className = "color-grid",
    disabled = false,
    asMenuItems = false,
    onselect,
  }: ColorSwatchesPropsType = $props();
</script>

<div class={className}>
  {#each CARD_COLORS as color (color)}
    {#if asMenuItems}
      <ContextMenuPrimitive.Item closeOnSelect={false} {disabled} onSelect={() => onselect(color)}>
        {#snippet child({ props })}
          <button
            {...props}
            class="color-swatch"
            class:none={!color}
            class:active={(value ?? "") === color}
            type="button"
            title={color || t.colour_none()}
            aria-label={color || t.colour_none()}
            aria-pressed={(value ?? "") === color}
            style={color ? `background:${color}` : ""}
            {disabled}
          >
            {#if !color}<X size={14} />{/if}
          </button>
        {/snippet}
      </ContextMenuPrimitive.Item>
    {:else}
      <button
        class="color-swatch"
        class:none={!color}
        class:active={(value ?? "") === color}
        type="button"
        title={color || t.colour_none()}
        aria-label={color || t.colour_none()}
        aria-pressed={(value ?? "") === color}
        style={color ? `background:${color}` : ""}
        {disabled}
        onclick={() => onselect(color)}
      >
        {#if !color}<X size={14} />{/if}
      </button>
    {/if}
  {/each}
</div>
