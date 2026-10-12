<script lang="ts">
  import FormError from "../ui/FormError.svelte";
  import { GuardAction, SteamGuardField } from "../../lib/steam-guard/steam-guard.constants";
  import { t, relativeDate } from "../../lib/i18n/i18n.svelte";

  import ActionButton from "../ui/ActionButton.svelte";
  import type { SteamGuardControllerType } from "../../lib/steam-guard/steam-guard.svelte";
  import type { AccountType } from "../../lib/types";
  import { Download, Lock, RefreshCw, ShieldOff, Trash2, UserRound } from "@lucide/svelte";
  import RevocationCode from "./RevocationCode.svelte";
  type SteamGuardSecurityPropsType = { controller: SteamGuardControllerType; entry: AccountType };

  let { controller, entry }: SteamGuardSecurityPropsType = $props();
</script>

<section class="steamguard-card">
  <h3 class="block-title"><UserRound size={15} /> {t.profile_title()}</h3>
  <p class="steamguard-note">
    {entry.profile_fetched_at
      ? t.profile_fetched({ when: relativeDate(entry.profile_fetched_at) })
      : t.profile_never_fetched()}
  </p>
  <div class="steamguard-row">
    <button class="button" type="button" disabled={controller.busy} onclick={controller.refresh}
      ><RefreshCw size={15} /> {t.profile_refresh_action()}</button
    >
  </div>
</section>

<section class="steamguard-card">
  <h3 class="block-title"><Lock size={15} /> {t.guard_security_title()}</h3>
  <p class="steamguard-note">
    {t.guard_security_recovery_hint()}
  </p>
  <div class="fact revocation-fact">
    <span class="fact-label">{t.guard_revocation_label()}</span>
    <RevocationCode {entry} />
  </div>
  <div class="steamguard-row">
    <button class="button" type="button" disabled={controller.busy} onclick={controller.exportThis}
      ><Download size={15} /> {t.backup_export_account_action()}</button
    >
  </div>
  <p class="steamguard-note">
    {t.guard_security_remove_warning()}
  </p>
  <div class="steamguard-row">
    <ActionButton
      icon={ShieldOff}
      label={t.guard_remove_action()}
      busyLabel={t.common_removing()}
      busy={controller.busyAction === GuardAction.REMOVE}
      disabled={controller.busy}
      className="button danger"
      onclick={controller.removeAuthenticator}
    />
  </div>
  <p class="steamguard-note">
    {t.guard_forget_description()}
  </p>
  <div class="steamguard-row">
    <button
      class="button danger"
      type="button"
      disabled={controller.busy}
      onclick={controller.forget}><Trash2 size={15} /> {t.guard_forget_action()}</button
    >
  </div>
  <FormError message={controller.errorFor(SteamGuardField.SECURITY)} />
</section>
