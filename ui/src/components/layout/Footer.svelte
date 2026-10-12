<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { openRepository } from "../../lib/actions/actions.svelte";
  import { app, vault } from "../../lib/state/state.svelte";
  import { createLockCountdown } from "../../lib/vault/lock-countdown.svelte";
  import { AlertTriangle, Lock, LockOpen, ShieldOff } from "@lucide/svelte";

  const status = $derived(vault());
  const countdown = createLockCountdown();
  const label = $derived(
    status.error
      ? t.footer_vault_error()
      : status.locked
        ? t.footer_vault_locked()
        : status.encrypted
          ? t.footer_vault_unlocked()
          : t.footer_vault_unprotected(),
  );
  const Icon = $derived(
    status.error ? AlertTriangle : status.locked ? Lock : status.encrypted ? LockOpen : ShieldOff,
  );
</script>

<footer class="footer">
  <button
    class="version"
    type="button"
    title="{app.info.name} {app.info.version} {t.footer_project_link()}"
    onclick={openRepository}
  >
    v{app.info.version}
  </button>
  <span class="footer-count">{t.accounts_count({ count: status.count })}</span>
  <button
    class="vault-state"
    class:locked={status.locked}
    class:warning={Boolean(status.error) || (!status.encrypted && !status.locked)}
    type="button"
    title={status.locked ? t.vault_unlock_action() : t.vault_settings_action()}
    onclick={() => (app.vaultOpen = true)}
  >
    <Icon size={14} />
    <span>{label}</span>
    {#if !status.locked && countdown.label}<span class="vault-state-countdown"
        >· {countdown.label}</span
      >{/if}
  </button>
</footer>
