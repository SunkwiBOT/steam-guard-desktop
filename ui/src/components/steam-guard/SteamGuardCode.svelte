<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import CopyButton from "../ui/CopyButton.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { liveCode, watchCode } from "../../lib/steam-guard/codes.svelte";

  type SteamGuardCodePropsType = { steamId: string };

  let { steamId }: SteamGuardCodePropsType = $props();
  const code = $derived(liveCode(steamId));
  const paused = $derived(app.idle);

  $effect(() => watchCode(steamId));
</script>

{#if code?.code}
  <div class="steamguard-code" class:paused>
    <span class="steamguard-code-value" aria-hidden={paused}>{code.code}</span>
    <CopyButton
      {steamId}
      copyKey={`guard:${steamId}`}
      label={t.guard_code_copy_label()}
      title={t.guard_code_copy_tooltip()}
      disabled={paused}
      action={() => copySteamGuardCode(steamId)}
    />
  </div>
  <div class="steamguard-timer" class:paused aria-hidden="true">
    <span style={`width:${((code.remaining / code.period) * 100).toFixed(1)}%`}></span>
  </div>
  <p class="steamguard-note">
    {paused ? t.guard_code_paused() : t.guard_code_countdown({ seconds: code.remaining })}
  </p>
{:else}
  <p class="steamguard-note">
    {(code?.error && errorMessage(code.error)) ||
      (paused ? t.guard_code_hidden() : t.guard_code_loading())}
  </p>
{/if}
