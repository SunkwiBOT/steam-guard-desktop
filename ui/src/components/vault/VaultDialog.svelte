<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";

  import { Lock, X } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import VaultControls from "./VaultControls.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
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

<AppDialog open={app.vaultOpen} onclose={close}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
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
  </DialogContent>
</AppDialog>
