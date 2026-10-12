<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { focusTrap } from "../../lib/actions/focus-trap";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { Lock, X } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import VaultControls from "./VaultControls.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  let busy = $state(false);
  const close = (): void => {
    if (!busy) {
      app.vaultOpen = false;
      app.vaultCopySteamId = null;
    }
  };

  const ready = async (): Promise<void> => {
    const steamId = app.vaultCopySteamId;
    app.vaultOpen = false;
    app.vaultCopySteamId = null;
    if (steamId) await copySteamGuardCode(steamId);
  };
  const uid = $props.id();
</script>

{#if app.vaultOpen}
  <div class="modal vault-modal">
    <div class="modal-backdrop"></div>
    <div
      class="dialog small"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      use:focusTrap={{ onEscape: close }}
    >
      <DialogHeader
        id={`${uid}-title`}
        title={app.snapshot?.vault.locked ? t.vault_unlock_title() : t.vault_title()}
        icon={Lock}
        onclose={close}
        closeLabel={t.vault_close()}
        closeDisabled={busy}
      />
      <VaultControls onready={ready} onbusy={(value) => (busy = value)} showSettings />
      <footer class="dialog-foot">
        <button class="button" type="button" disabled={busy} onclick={close}
          ><X size={15} />
          {app.snapshot?.vault.locked ? t.common_cancel() : t.common_close()}</button
        >
      </footer>
    </div>
  </div>
{/if}
