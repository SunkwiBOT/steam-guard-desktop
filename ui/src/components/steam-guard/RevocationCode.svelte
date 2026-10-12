<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { untrack } from "svelte";
  import { Eye, EyeOff } from "@lucide/svelte";
  import { showToast } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import type { AccountType } from "../../lib/types";

  type RevocationCodePropsType = { entry: AccountType };

  let { entry }: RevocationCodePropsType = $props();
  const selectedId = $derived(entry.steam_id);
  let revealed = $state("");
  let busy = $state(false);
  let generation = 0;

  //! Hidden again whenever the account changes: the code is a secret.
  $effect(() => {
    void selectedId;
    untrack(() => {
      generation += 1;
      revealed = "";
      busy = false;
    });
  });

  const read = (): Promise<string> =>
    invoke("steam_guard_revocation_code", { steamId: entry.steam_id });

  const toggle = async (): Promise<void> => {
    if (busy) return;
    if (revealed) {
      revealed = "";
      return;
    }
    const current = generation;
    busy = true;
    try {
      const value = await read();
      if (current === generation) revealed = value;
    } catch (cause) {
      if (current === generation) showToast(errorMessage(cause), false, entry);
    } finally {
      if (current === generation) busy = false;
    }
  };
  const toggleLabel = $derived(
    revealed
      ? t.common_hide_value({ value: t.guard_revocation_value() })
      : t.common_show_value({ value: t.guard_revocation_value() }),
  );
</script>

<span class="fact-row">
  <span class="fact-value" class:accent={Boolean(revealed)}>{revealed || "••••••"}</span>
  <button
    class="copy-button"
    type="button"
    title={toggleLabel}
    aria-label={toggleLabel}
    aria-pressed={Boolean(revealed)}
    aria-busy={busy}
    disabled={busy}
    onclick={toggle}
  >
    {#if busy}<AsyncIcon size={15} />{:else if revealed}<EyeOff size={15} />{:else}<Eye
        size={15}
      />{/if}
  </button>
  <CopyButton
    steamId={entry.steam_id}
    copyKey={`revocation:${entry.steam_id}`}
    label={t.clipboard_copy_label({ value: t.guard_revocation_value() })}
    valueLabel={t.guard_revocation_label()}
    {read}
    disabled={busy}
  />
</span>
