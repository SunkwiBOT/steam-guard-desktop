<script lang="ts">
  import { Dialog } from "$lib/components/ui/dialog/index.js";
  import type { Snippet } from "svelte";

  // ! Mounted only while open (bits-ui's Portal appends to `body` at mount
  // time, not at `open`) and bound two-way: when bits-ui closes itself
  // (Escape, outside click) the getter is re-read right after `onclose`, so a
  // veto that leaves `open` true reopens the dialog instead of leaving it
  // hidden while the app still thinks it is open.
  type AppDialogPropsType = { open: boolean; onclose: () => void; children: Snippet };

  let { open, onclose, children }: AppDialogPropsType = $props();
</script>

{#if open}
  <Dialog
    bind:open={
      () => open,
      (next) => {
        if (!next) onclose();
      }
    }
  >
    {@render children()}
  </Dialog>
{/if}
