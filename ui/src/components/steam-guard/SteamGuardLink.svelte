<script lang="ts">
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import ActionButton from "../ui/ActionButton.svelte";
  import type { GuardFlowType } from "../../lib/steam-guard/steam-guard-flow";
  import type { ToastAccountType } from "../../lib/types";
  import { AlertTriangle, ArrowRightLeft, ShieldOff, Check } from "@lucide/svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import SteamGuardIcon from "./SteamGuardIcon.svelte";
  import FormError from "../ui/FormError.svelte";
  import {
    GuardAction,
    LinkConfirmKind,
    LinkStatus,
    SteamGuardField,
  } from "../../lib/steam-guard/steam-guard.constants";
  type SteamGuardLinkPropsType = { flow: GuardFlowType; account: ToastAccountType | null };

  const linkMessages: Partial<Record<LinkStatus, () => string>> = {
    [LinkStatus.RESUME]: () => t.guard_link_resume(),
    [LinkStatus.NEEDS_PHONE]: () => t.guard_link_phone_required(),
    [LinkStatus.NEEDS_EMAIL]: () => t.guard_link_email_required(),
  };

  const linkMessage = (status: LinkStatus): string =>
    (linkMessages[status] ?? (() => t.common_unknown()))();
  let { flow, account }: SteamGuardLinkPropsType = $props();
  const name = $derived(account?.display_name || flow.link?.account_name || "");
  const uid = $props.id();
</script>

{#if flow.link}
  <section class="steamguard-card">
    <h3 class="block-title"><SteamGuardIcon size={17} /> {t.guard_link_title()}</h3>
    <p class="steamguard-note">
      <MessageText content={rich(t.guard_link_description, { account: name })} {account} />
    </p>
    {#if flow.link.status === LinkStatus.LINKED || flow.link.status === LinkStatus.RESUME}
      <p class="steamguard-warning">
        <AlertTriangle size={15} />
        {t.guard_link_recovery_hint()}
        <strong>{flow.link.revocation_code}</strong>
      </p>
      <p class="steamguard-note">
        {#if flow.link.status === LinkStatus.RESUME}
          {linkMessage(flow.link.status)}
        {:else if flow.link.confirm_kind === LinkConfirmKind.EMAIL}
          {t.guard_link_email_sent()}
        {:else}
          <MessageText
            content={rich(t.guard_link_sms_sent, { hint: flow.link.phone_hint || "…" })}
          />
        {/if}
      </p>
      <div class="steamguard-row">
        <div class="steamguard-field">
          <input
            class="input"
            placeholder={t.guard_link_code_label()}
            aria-label={t.guard_link_code_label()}
            aria-invalid={Boolean(flow.errorFor(SteamGuardField.LINK_CODE))}
            aria-describedby={flow.errorFor(SteamGuardField.LINK_CODE)
              ? `${uid}-code-error`
              : undefined}
            bind:value={flow.linkCode}
            disabled={flow.busy}
            onkeydown={(event) => event.key === "Enter" && flow.finalizeLink()}
          />
          <FormError id={`${uid}-code-error`} message={flow.errorFor(SteamGuardField.LINK_CODE)} />
        </div>
        <ActionButton
          icon={Check}
          label={t.common_finish()}
          busyLabel={t.common_finishing()}
          busy={flow.busyAction === GuardAction.FINALIZE}
          disabled={flow.busy}
          className="button primary"
          onclick={flow.finalizeLink}
        />
      </div>
    {:else if flow.link.status === LinkStatus.AUTHENTICATOR_PRESENT}
      <p class="steamguard-note">
        {t.guard_link_existing_description()}
      </p>
      {#if !flow.transferStarted}
        <div class="steamguard-row">
          <ActionButton
            icon={ArrowRightLeft}
            label={t.guard_transfer_start_action()}
            busyLabel={t.guard_transfer_start_pending()}
            busy={flow.busyAction === GuardAction.TRANSFER_START}
            disabled={flow.busy}
            className="button primary"
            onclick={flow.startTransfer}
          />
        </div>
        <FormError message={flow.errorFor(SteamGuardField.TRANSFER)} />
      {:else}
        <div class="steamguard-row">
          <div class="steamguard-field">
            <input
              class="input"
              placeholder={t.guard_transfer_code_label()}
              aria-label={t.guard_transfer_code_label()}
              aria-invalid={Boolean(flow.errorFor(SteamGuardField.LINK_CODE))}
              aria-describedby={flow.errorFor(SteamGuardField.LINK_CODE)
                ? `${uid}-code-error`
                : undefined}
              bind:value={flow.linkCode}
              disabled={flow.busy}
              onkeydown={(event) => event.key === "Enter" && flow.finishTransfer()}
            />
            <FormError
              id={`${uid}-code-error`}
              message={flow.errorFor(SteamGuardField.LINK_CODE)}
            />
          </div>
          <ActionButton
            icon={ArrowRightLeft}
            label={t.guard_transfer_action()}
            busyLabel={t.guard_transfer_pending()}
            busy={flow.busyAction === GuardAction.TRANSFER_FINISH}
            disabled={flow.busy}
            className="button primary"
            onclick={flow.finishTransfer}
          />
        </div>
      {/if}
      <div class="steamguard-row">
        <div class="steamguard-field">
          <PasswordInput
            label={t.guard_revocation_label()}
            placeholder={t.guard_revocation_placeholder()}
            bind:value={flow.revocationInput}
            disabled={flow.busy}
            invalid={Boolean(flow.errorFor(SteamGuardField.REVOCATION))}
            describedby={flow.errorFor(SteamGuardField.REVOCATION)
              ? `${uid}-revocation-error`
              : undefined}
          />
          <FormError
            id={`${uid}-revocation-error`}
            message={flow.errorFor(SteamGuardField.REVOCATION)}
          />
        </div>
        <ActionButton
          icon={ShieldOff}
          label={t.guard_revoke_action()}
          busyLabel={t.guard_revoke_pending()}
          busy={flow.busyAction === GuardAction.REVOKE}
          disabled={flow.busy}
          className="button danger"
          onclick={flow.revokeDuringLink}
        />
      </div>
    {:else}
      <p class="steamguard-note">{linkMessage(flow.link.status)}</p>
    {/if}
    <FormError message={flow.errorFor(SteamGuardField.LINK)} />
  </section>
{/if}
