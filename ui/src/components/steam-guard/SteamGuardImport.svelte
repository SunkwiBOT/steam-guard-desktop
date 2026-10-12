<script lang="ts">
  import MessageText from "../ui/MessageText.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import { GuardAction, SteamGuardField } from "../../lib/steam-guard/steam-guard.constants";
  import ActionButton from "../ui/ActionButton.svelte";
  import type { AddAccountControllerType } from "../../lib/steam-guard/add-account.svelte";
  import { FileSearch, Upload } from "@lucide/svelte";
  import PasswordInput from "../ui/PasswordInput.svelte";
  import FormError from "../ui/FormError.svelte";
  type SteamGuardImportPropsType = { controller: AddAccountControllerType };

  let { controller }: SteamGuardImportPropsType = $props();
  const uid = $props.id();
</script>

<section class="steamguard-card">
  <h3 class="block-title">
    <Upload size={15} />
    {t.guard_import_title()}
  </h3>
  <p class="steamguard-note">
    <MessageText
      content={rich(t.guard_import_description, { format: ".maFile", manifest: "manifest.json" }, [
        "format",
        "manifest",
      ])}
    />
  </p>
  <div class="steamguard-row">
    <div class="steamguard-field">
      <input
        class="input"
        readonly
        aria-label={t.guard_import_file_label()}
        aria-invalid={Boolean(controller.errorFor(SteamGuardField.IMPORT_FILE))}
        aria-describedby={controller.errorFor(SteamGuardField.IMPORT_FILE)
          ? `${uid}-file-error`
          : undefined}
        placeholder={t.common_file_empty()}
        value={controller.importPath}
      />
      <FormError
        id={`${uid}-file-error`}
        message={controller.errorFor(SteamGuardField.IMPORT_FILE)}
      />
    </div>
    <ActionButton
      icon={FileSearch}
      label={t.common_browse()}
      busyLabel={t.common_opening()}
      busy={controller.busyAction === GuardAction.BROWSE}
      disabled={controller.busy}
      className="button"
      onclick={controller.pickFile}
    />
  </div>
  {#if controller.passkeyNeeded}
    <div class="steamguard-row">
      <div class="steamguard-field">
        <PasswordInput
          label={t.guard_import_password_label()}
          placeholder={t.guard_import_password_placeholder()}
          bind:value={controller.importPasskey}
          disabled={controller.busy}
          invalid={Boolean(controller.errorFor(SteamGuardField.IMPORT_PASSWORD))}
          describedby={controller.errorFor(SteamGuardField.IMPORT_PASSWORD)
            ? `${uid}-password-error`
            : undefined}
          onkeydown={(event) => event.key === "Enter" && controller.runImport()}
        />
        <FormError
          id={`${uid}-password-error`}
          message={controller.errorFor(SteamGuardField.IMPORT_PASSWORD)}
        />
      </div>
    </div>
  {/if}
  <div class="steamguard-actions">
    <ActionButton
      icon={Upload}
      label={t.common_import()}
      busyLabel={t.common_importing()}
      busy={controller.busyAction === GuardAction.IMPORT}
      disabled={controller.busy}
      className="button primary"
      onclick={controller.runImport}
    />
  </div>
  <FormError message={controller.errorFor(SteamGuardField.IMPORT)} />
</section>
