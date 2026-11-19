<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { t, rich } from "../../lib/i18n/i18n.svelte";

  import { ConfirmKind } from "../../lib/state/dialogs.constants";
  import { cancelConfirm, confirmDialog, runConfirmAction } from "../../lib/state/dialogs.svelte";
  import { accountById } from "../../lib/state/state.svelte";

  import MessageText from "../ui/MessageText.svelte";
  import { containsAccountName } from "../../lib/i18n/messages";
  import { AlertTriangle, Check, Trash2, X } from "@lucide/svelte";
  import AppDialog from "./AppDialog.svelte";
  import ActionButton from "../ui/ActionButton.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";

  const entry = $derived(accountById(confirmDialog.steamId ?? ""));
  const inlineAccount = $derived(
    entry && containsAccountName(confirmDialog.content, entry.display_name),
  );

  const ActionIcon = $derived(
    {
      [ConfirmKind.REMOVE]: Trash2,
      [ConfirmKind.CONFIRM]: confirmDialog.danger ? AlertTriangle : Check,
    }[confirmDialog.kind],
  );

  let checked = $state(false);

  $effect(() => {
    if (confirmDialog.open) {
      checked = false;
    }
  });
  const uid = $props.id();
</script>

<AppDialog open={confirmDialog.open} onclose={cancelConfirm}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader
      id={`${uid}-title`}
      title={confirmDialog.title}
      icon={ActionIcon}
      closeDisabled={confirmDialog.busy}
    />
    {#if entry && !inlineAccount}<p class="dialog-text">
        <MessageText
          content={rich(t.account_details_description, { account: entry.display_name })}
          account={entry}
        />
      </p>{/if}
    <p class="dialog-text"><MessageText content={confirmDialog.content} account={entry} /></p>
    {#if confirmDialog.checkboxLabel}
      <label class="checkbox">
        <Checkbox bind:checked disabled={confirmDialog.busy} />
        <span>{confirmDialog.checkboxLabel}</span>
      </label>
    {/if}
    <footer class="dialog-foot">
      <!-- svelte-ignore a11y_autofocus -->
      <button
        class="button"
        type="button"
        autofocus
        disabled={confirmDialog.busy}
        onclick={cancelConfirm}><X size={15} /> {t.common_cancel()}</button
      >
      <ActionButton
        icon={ActionIcon}
        label={confirmDialog.confirmLabel}
        busyLabel={t.common_working()}
        busy={confirmDialog.busy}
        disabled={confirmDialog.checkboxRequired && !checked}
        className={confirmDialog.danger ? "button danger" : "button primary"}
        onclick={() => void runConfirmAction({ checked })}
      />
    </footer>
  </DialogContent>
</AppDialog>
