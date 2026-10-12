<script lang="ts">
  import type { ToastAccountType } from "../../lib/types";
  import Avatar from "../ui/Avatar.svelte";
  type AccountCaptionPropsType = {
    entry: ToastAccountType;
    size?: number;
    inline?: boolean;
    bold?: boolean;
  };

  let { entry, size = 28, inline = false, bold = false }: AccountCaptionPropsType = $props();
</script>

<span class="account-caption" class:inline style:--avatar-size={`${size}px`}
  ><Avatar {entry} {size} /><span title={entry.display_name}
    >{#if bold}<strong>{entry.display_name}</strong>{:else}{entry.display_name}{/if}</span
  ></span
>
