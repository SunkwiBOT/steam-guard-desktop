<script lang="ts">
  import { app } from "../../lib/state/state.svelte";
</script>

{#if app.busy.active}
  <div class="busy" role="status" aria-live="polite" tabindex="-1">
    <div class="spinner"></div>
    <p>{app.busy.text}</p>
  </div>
{/if}
