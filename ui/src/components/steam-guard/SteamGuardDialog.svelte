<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import { focusTrap } from "../../lib/actions/focus-trap";
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import {
    GuardAction,
    SteamGuardScreen,
    SteamGuardField,
  } from "../../lib/steam-guard/steam-guard.constants";
  import {
    ChevronDown,
    Folder,
    Hourglass,
    KeyRound,
    Lock,
    Palette,
    Play,
    Settings,
    X,
  } from "@lucide/svelte";
  import { app, folderById, folders } from "../../lib/state/state.svelte";
  import { moveToFolder, setAccountColor } from "../../lib/actions/actions.svelte";
  import { dropdown, openDropdown } from "../../lib/state/menus.svelte";
  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import { createSteamGuardController } from "../../lib/steam-guard/steam-guard.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import FormError from "../ui/FormError.svelte";
  import SteamGuardCode from "./SteamGuardCode.svelte";
  import SteamGuardIcon from "./SteamGuardIcon.svelte";
  import AccountIdentity from "../accounts/AccountIdentity.svelte";
  import SteamIdentifiers from "../accounts/SteamIdentifiers.svelte";
  import VaultControls from "../vault/VaultControls.svelte";
  import SteamGuardConfirmations from "./SteamGuardConfirmations.svelte";
  import SteamGuardSecurity from "./SteamGuardSecurity.svelte";
  import SteamGuardLogin from "./SteamGuardLogin.svelte";
  import SteamGuardLink from "./SteamGuardLink.svelte";

  const controller = createSteamGuardController();
  const entry = $derived(controller.entry);
  const folderName = $derived(
    (entry?.folder_id && folderById(entry.folder_id)?.name) || t.folder_none(),
  );
  let folderButton = $state<HTMLButtonElement | null>(null);

  const pickFolder = (): void => {
    if (!entry || !folderButton) return;
    const account = entry;
    openDropdown({
      anchor: folderButton,
      value: account.folder_id ?? "",
      options: [
        { value: "", label: t.folder_none() },
        ...folders().map((item) => ({ value: item.id, label: item.name })),
      ],
      onSelect: (value) => void moveToFolder(account.steam_id, value || null),
    });
  };
  const uid = $props.id();
</script>

{#if app.steamGuardOpen && entry}
  <div class="modal" inert={app.vaultOpen}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-backdrop" onclick={controller.close}></div>
    <div
      class="dialog steamguard-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${uid}-title`}
      tabindex="-1"
      use:focusTrap={{ onEscape: controller.close }}
    >
      <DialogHeader
        id={`${uid}-title`}
        title={entry.display_name}
        icon={SteamGuardIcon}
        onclose={controller.close}
        closeDisabled={controller.busy && !controller.waiting}
        onback={controller.vaultReady && controller.screen !== SteamGuardScreen.HOME
          ? controller.back
          : undefined}
        backDisabled={controller.busy && !controller.waiting}
        backBusy={controller.busyAction === GuardAction.CANCEL_LOGIN ||
          controller.busyAction === GuardAction.CANCEL_LINK}
      />
      {#if controller.vaultReady && controller.screen === SteamGuardScreen.HOME}
        <div class="account-facts">
          <div class="fact">
            <span class="fact-label">{t.guard_login_name_label()}</span>
            <AccountIdentity {entry} />
          </div>
          <SteamIdentifiers steamId={entry.steam_id} />
        </div>
      {/if}

      <div class="dialog-body">
        {#if controller.vault.error}
          <p class="steamguard-note">
            <MessageText
              content={rich(t.guard_vault_unavailable_description, { account: entry.display_name })}
              account={entry}
            />
          </p>
          <p class="steamguard-error">
            {errorMessage(controller.vault.error)}
            {t.vault_restore_hint()}
          </p>
        {:else if controller.vault.locked}
          <section class="steamguard-card">
            <h3 class="block-title">
              <Lock size={15} />
              {t.vault_unlock_title()}
            </h3>
            <p class="steamguard-note">
              <MessageText
                content={rich(t.guard_vault_unlock_description, { account: entry.display_name })}
                account={entry}
              />
            </p>
            <VaultControls onbusy={(value) => (controller.busy = value)} />
          </section>
        {:else if controller.screen === SteamGuardScreen.HOME}
          {#if entry.pending}
            <section class="steamguard-card">
              <h3 class="block-title"><Hourglass size={15} /> {t.guard_pending_title()}</h3>
              <p class="steamguard-note">
                <MessageText
                  content={rich(t.guard_pending_description, { account: entry.display_name })}
                  account={entry}
                />
              </p>
              <div class="steamguard-actions">
                <ActionButton
                  icon={Play}
                  label={t.guard_pending_resume_action()}
                  busyLabel={t.guard_link_pending()}
                  busy={controller.busyAction === GuardAction.LINK}
                  disabled={controller.busy || !controller.vaultReady}
                  className="button primary"
                  onclick={controller.resumeActivation}
                />
              </div>
            </section>
            <section class="steamguard-card">
              <h3 class="block-title"><Lock size={15} /> {t.guard_security_title()}</h3>
              <p class="steamguard-note">{t.guard_forget_description()}</p>
              <div class="steamguard-row">
                <button
                  class="button danger"
                  type="button"
                  disabled={controller.busy}
                  onclick={controller.forget}><X size={15} /> {t.guard_forget_action()}</button
                >
              </div>
            </section>
          {:else}
            <section class="steamguard-card">
              <h3 class="block-title"><KeyRound size={15} /> {t.guard_code_label()}</h3>
              <p class="steamguard-note">
                <MessageText
                  content={rich(t.guard_code_description, { account: entry.display_name })}
                  account={entry}
                />
              </p>
              <SteamGuardCode steamId={entry.steam_id} />
            </section>

            <SteamGuardConfirmations {controller} {entry} />
            <SteamGuardSecurity {controller} {entry} />
          {/if}
        {:else if controller.screen === SteamGuardScreen.LOGIN}
          <SteamGuardLogin flow={controller.flow} account={entry} />
        {:else if controller.screen === SteamGuardScreen.LINK && controller.link}
          <SteamGuardLink flow={controller.flow} account={entry} />
        {/if}
        {#if !controller.vault.error && controller.screen === SteamGuardScreen.HOME}
          <!-- ! Folder and colour live in the settings database, not the vault:
               they stay editable while the vault is locked. -->
          <section class="steamguard-card account-row">
            <div class="account-block">
              <h3 class="block-title"><Folder size={15} /> {t.folder_label()}</h3>
              <button
                class="select-button"
                type="button"
                aria-label={t.folder_label()}
                aria-haspopup="listbox"
                aria-expanded={dropdown.open && dropdown.anchor === folderButton}
                bind:this={folderButton}
                disabled={controller.busy}
                onclick={pickFolder}
              >
                <span>{folderName}</span>
                <span class="picker-caret"><ChevronDown size={14} /></span>
              </button>
            </div>
            <div class="account-block">
              <h3 class="block-title"><Palette size={15} /> {t.account_colour()}</h3>
              <ColorSwatches
                value={entry.color}
                disabled={controller.busy}
                onselect={(color) => void setAccountColor(entry.steam_id, color)}
              />
            </div>
          </section>
        {/if}
        <FormError message={controller.errorFor(SteamGuardField.GENERAL)} />
      </div>

      <footer class="dialog-foot">
        <button
          class="button"
          type="button"
          disabled={controller.busy || controller.waiting}
          onclick={controller.openVaultSettings}
          ><Settings size={15} /> {t.vault_settings_action()}</button
        >
        <button
          class="button"
          type="button"
          disabled={controller.busy && !controller.waiting}
          onclick={controller.close}><X size={15} /> {t.common_close()}</button
        >
      </footer>
    </div>
  </div>
{/if}
