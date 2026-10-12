<script lang="ts">
  import { t, rich } from "../../lib/i18n/i18n.svelte";
  import MessageText from "../ui/MessageText.svelte";

  import {
    GuardAction,
    LoginPurpose,
    SteamGuardField,
    GuardKind,
  } from "../../lib/steam-guard/steam-guard.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import type { GuardFlowType } from "../../lib/steam-guard/steam-guard-flow";
  import type { ToastAccountType } from "../../lib/types";
  import { ArrowRight, LogIn, QrCode, X } from "@lucide/svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import SteamGuardCode from "./SteamGuardCode.svelte";
  import FormError from "../ui/FormError.svelte";
  type SteamGuardLoginPropsType = { flow: GuardFlowType; account: ToastAccountType | null };

  let { flow, account }: SteamGuardLoginPropsType = $props();
  const uid = $props.id();
</script>

<section class="steamguard-card">
  <h3 class="block-title">
    <LogIn size={15} />
    {t.guard_login_title()}
  </h3>
  {#if !flow.loginBegin}
    <p class="steamguard-note">
      {#if !account}
        {t.guard_login_add_description()}
      {:else if flow.loginPurpose === LoginPurpose.LINK}
        <MessageText
          content={rich(t.guard_login_link_description, { account: account.display_name })}
          {account}
        />
      {:else}
        <MessageText
          content={rich(t.guard_login_confirmations_description, {
            account: account.display_name,
          })}
          {account}
        />
      {/if}
    </p>
    <div class="steamguard-row">
      <div class="steamguard-field">
        <label class="field-label" for={`${uid}-name`}>{t.guard_login_name_label()}</label>
        <input
          id={`${uid}-name`}
          class="input"
          autocomplete="username"
          autocapitalize="off"
          spellcheck="false"
          placeholder={t.guard_login_name_placeholder()}
          readonly={flow.accountNameLocked}
          disabled={flow.busy}
          aria-invalid={Boolean(flow.errorFor(SteamGuardField.ACCOUNT_NAME))}
          aria-describedby={flow.errorFor(SteamGuardField.ACCOUNT_NAME)
            ? `${uid}-name-error`
            : undefined}
          bind:value={flow.accountName}
          onkeydown={(event) => event.key === "Enter" && flow.startPassword()}
        />
        <FormError id={`${uid}-name-error`} message={flow.errorFor(SteamGuardField.ACCOUNT_NAME)} />
      </div>
    </div>
    <div class="steamguard-row">
      <div class="steamguard-field">
        <label class="field-label" for={`${uid}-password`}>{t.guard_login_password_label()}</label>
        <PasswordInput
          id={`${uid}-password`}
          label={t.guard_login_password_label()}
          placeholder={t.common_password()}
          autocomplete="current-password"
          bind:value={flow.password}
          disabled={flow.busy}
          invalid={Boolean(flow.errorFor(SteamGuardField.PASSWORD))}
          describedby={flow.errorFor(SteamGuardField.PASSWORD)
            ? `${uid}-password-error`
            : undefined}
          onkeydown={(event) => event.key === "Enter" && flow.startPassword()}
        />
        <FormError id={`${uid}-password-error`} message={flow.errorFor(SteamGuardField.PASSWORD)} />
      </div>
    </div>
    <div class="steamguard-actions">
      <ActionButton
        icon={LogIn}
        label={t.guard_login_action()}
        busyLabel={t.guard_login_pending()}
        busy={flow.busyAction === GuardAction.PASSWORD}
        disabled={flow.busy}
        className="button primary"
        onclick={flow.startPassword}
      />
      <ActionButton
        icon={QrCode}
        label={t.guard_login_qr_action()}
        busyLabel={t.guard_login_qr_pending()}
        busy={flow.busyAction === GuardAction.QR}
        disabled={flow.busy}
        className="button"
        onclick={flow.startQr}
      />
    </div>
  {:else if flow.loginBegin.qr_svg}
    <p class="steamguard-note">
      {#if account}
        <MessageText
          content={rich(t.guard_login_finish_description, { account: account.display_name })}
          {account}
        />
      {:else}
        {t.guard_login_qr_description()}
      {/if}
    </p>
    {#if account}<p class="steamguard-note">{t.guard_login_qr_description()}</p>{/if}
    <div class="steamguard-qr">{@html flow.loginBegin.qr_svg}</div>
    {#if flow.waiting}
      <p class="steamguard-note">{t.guard_login_approval_pending()}</p>
    {:else}
      <p class="steamguard-note">{t.guard_login_qr_expired()}</p>
    {/if}
    <ActionButton
      icon={X}
      label={t.guard_login_cancel()}
      busyLabel={t.common_cancelling()}
      busy={flow.busyAction === GuardAction.CANCEL_LOGIN}
      onclick={flow.cancelLogin}
    />
  {:else}
    <p class="steamguard-note">
      {#if account}
        <MessageText
          content={rich(t.guard_login_finish_description, { account: account.display_name })}
          {account}
        />
      {:else}
        {t.guard_login_finish_new_description()}
      {/if}
    </p>
    <p class="steamguard-note">
      {#if flow.needsExternalConfirmation()}
        {flow.selectedGuard === GuardKind.EMAIL_CONFIRMATION
          ? t.guard_login_email_approval_hint()
          : t.guard_login_device_approval_hint()}
      {:else if flow.guardNeedsCode}
        {flow.selectedGuard === GuardKind.EMAIL_CODE
          ? t.guard_login_email_code_hint()
          : t.guard_login_device_code_hint()}
      {:else}
        {t.guard_login_continue_hint()}
      {/if}
    </p>
    {#if flow.selectedGuard === GuardKind.DEVICE_CODE && flow.localCodeSteamId}
      <SteamGuardCode steamId={flow.localCodeSteamId} />
      <p class="steamguard-note">
        {t.guard_login_saved_code_hint()}
      </p>
    {/if}
    {#if flow.guardNeedsCode}
      <div class="steamguard-row">
        <div class="steamguard-field">
          <input
            class="input"
            placeholder={t.guard_code_label()}
            aria-label={t.guard_code_label()}
            aria-invalid={Boolean(flow.errorFor(SteamGuardField.LOGIN_CODE))}
            aria-describedby={flow.errorFor(SteamGuardField.LOGIN_CODE)
              ? `${uid}-code-error`
              : undefined}
            bind:value={flow.loginCode}
            disabled={flow.busy || flow.waiting}
            onkeydown={(event) => event.key === "Enter" && flow.submitPasswordCode()}
          />
          <FormError id={`${uid}-code-error`} message={flow.errorFor(SteamGuardField.LOGIN_CODE)} />
        </div>
      </div>
    {/if}
    <div class="steamguard-actions">
      <ActionButton
        icon={X}
        label={t.common_cancel()}
        busyLabel={t.common_cancelling()}
        busy={flow.busyAction === GuardAction.CANCEL_LOGIN}
        onclick={flow.cancelLogin}
      />
      <ActionButton
        icon={ArrowRight}
        label={t.common_continue()}
        busyLabel={flow.busyAction === GuardAction.LINK
          ? t.guard_link_pending()
          : flow.busyAction === GuardAction.LOGIN_CODE
            ? t.guard_code_generate_pending()
            : t.guard_login_pending()}
        busy={flow.busyAction === GuardAction.LOGIN ||
          flow.busyAction === GuardAction.LINK ||
          flow.busyAction === GuardAction.LOGIN_CODE ||
          flow.waiting}
        disabled={flow.busy || flow.waiting}
        className="button primary"
        onclick={flow.submitPasswordCode}
      />
    </div>
  {/if}
  <FormError message={flow.errorFor(SteamGuardField.LOGIN)} />
</section>
