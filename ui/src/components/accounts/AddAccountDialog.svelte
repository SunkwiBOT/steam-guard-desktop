<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { errorMessage, vaultErrorHint } from "../../lib/errors/errors.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import {
    GuardAction,
    SteamGuardScreen,
    SteamGuardField,
  } from "../../lib/steam-guard/steam-guard.constants";
  import { Archive, FileUp, Lock, Smartphone, UserPlus, X } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import { createAddAccountController } from "../../lib/steam-guard/add-account.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import FormError from "../ui/FormError.svelte";
  import VaultControls from "../vault/VaultControls.svelte";
  import SteamGuardImport from "../steam-guard/SteamGuardImport.svelte";
  import SteamGuardLogin from "../steam-guard/SteamGuardLogin.svelte";
  import SteamGuardLink from "../steam-guard/SteamGuardLink.svelte";

  const controller = createAddAccountController();
  const uid = $props.id();
</script>

<AppDialog open={app.addAccountOpen} onclose={controller.close}>
  <DialogContent
    showCloseButton={false}
    class="sm:max-w-2xl max-h-[86vh] flex flex-col overflow-hidden"
    aria-labelledby={`${uid}-title`}
  >
    <DialogHeader
      id={`${uid}-title`}
      title={t.account_add_title()}
      icon={UserPlus}
      onclose={controller.close}
      closeDisabled={controller.busy && !controller.waiting}
      onback={controller.vaultReady && controller.screen !== SteamGuardScreen.HOME
        ? controller.back
        : undefined}
      backDisabled={controller.busy && !controller.waiting}
      backBusy={controller.busyAction === GuardAction.CANCEL_LOGIN ||
        controller.busyAction === GuardAction.CANCEL_LINK}
    />

    <div class="dialog-body">
      {#if controller.vault.error}
        <p class="steamguard-error">
          {errorMessage(controller.vault.error)}
          {vaultErrorHint(controller.vault.error)}
        </p>
      {:else if controller.vault.locked}
        <section class="steamguard-card">
          <h3 class="block-title">
            <Lock size={15} />
            {t.vault_unlock_title()}
          </h3>
          <p class="steamguard-note">{t.account_add_vault_locked()}</p>
          <VaultControls onbusy={(value) => (controller.busy = value)} />
        </section>
      {:else if controller.screen === SteamGuardScreen.HOME}
        <p class="steamguard-note add-intro">{t.account_add_description()}</p>
        <div class="add-options">
          <button
            class="add-option"
            type="button"
            disabled={controller.busy}
            onclick={controller.beginLink}
          >
            <span class="add-option-icon"><Smartphone size={20} /></span>
            <span class="add-option-title">{t.account_add_link_title()}</span>
            <span class="add-option-text">{t.account_add_link_description()}</span>
          </button>
          <button
            class="add-option"
            type="button"
            disabled={controller.busy}
            onclick={controller.beginImport}
          >
            <span class="add-option-icon"><FileUp size={20} /></span>
            <span class="add-option-title"
              ><MessageText
                content={rich(t.account_add_mafile_title, { format: ".maFile" }, ["format"])}
              /></span
            >
            <span class="add-option-text">{t.account_add_mafile_description()}</span>
          </button>
          <button
            class="add-option"
            type="button"
            disabled={controller.busy}
            onclick={controller.importBackup}
          >
            <span class="add-option-icon"><Archive size={20} /></span>
            <span class="add-option-title"
              ><MessageText
                content={rich(
                  t.account_add_backup_title,
                  { format: `.${app.info.backup_extension}` },
                  ["format"],
                )}
              /></span
            >
            <span class="add-option-text">{t.account_add_backup_description()}</span>
          </button>
        </div>
      {:else if controller.screen === SteamGuardScreen.IMPORT}
        <SteamGuardImport {controller} />
      {:else if controller.screen === SteamGuardScreen.LOGIN}
        <SteamGuardLogin flow={controller.flow} account={null} />
      {:else if controller.screen === SteamGuardScreen.LINK && controller.link}
        <SteamGuardLink flow={controller.flow} account={null} />
      {/if}
      <FormError message={controller.errorFor(SteamGuardField.GENERAL)} />
    </div>

    <footer class="dialog-foot">
      <button
        class="button"
        type="button"
        disabled={controller.busy && !controller.waiting}
        onclick={controller.close}><X size={15} /> {t.common_close()}</button
      >
    </footer>
  </DialogContent>
</AppDialog>
