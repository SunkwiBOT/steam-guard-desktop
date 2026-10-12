<script lang="ts">
  import type { AvatarSourceType } from "../../lib/types";

  type AvatarPropsType = { entry: AvatarSourceType; size?: number; extra?: string };

  let { entry, size = 68, extra = "" }: AvatarPropsType = $props();
</script>

{#if entry.avatar}
  <img
    class="avatar {extra}"
    style="width:{size}px;height:{size}px"
    src={entry.avatar}
    alt=""
    draggable="false"
  />
{:else}
  <span class="avatar initial {extra}" aria-hidden="true" style="width:{size}px;height:{size}px">
    {entry.initial || "?"}
  </span>
{/if}
