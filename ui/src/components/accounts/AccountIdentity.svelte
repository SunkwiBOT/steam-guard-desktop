<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { untrack } from "svelte";
  import { Eye, EyeOff } from "@lucide/svelte";
  import { app, showToast } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import AsyncIcon from "../ui/AsyncIcon.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import type { AccountType } from "../../lib/types";

  type AccountIdentityPropsType = { entry: AccountType };

  let { entry }: AccountIdentityPropsType = $props();
  const hiddenGlobally = $derived(app.snapshot?.masked !== false);
  const selectedId = $derived(entry.steam_id);
  let visible = $state(false);
  let revealed = $state("");
  let busy = $state(false);
  let generation = 0;
  let previousKey = "";

  //! Follows the privacy setting again whenever the account or the setting changes.
  $effect(() => {
    const key = `${selectedId}:${hiddenGlobally}`;
    untrack(() => {
      if (key === previousKey) return;
      previousKey = key;
      generation += 1;
      visible = !hiddenGlobally;
      revealed = "";
      busy = false;
    });
  });

  const read = (): Promise<string> =>
    hiddenGlobally
      ? invoke("account_login_name", { steamId: entry.steam_id })
      : Promise.resolve(entry.account_name);

  const toggle = async (): Promise<void> => {
    if (busy) return;
    if (visible) {
      visible = false;
      revealed = "";
      return;
    }
    const current = generation;
    busy = true;
    try {
      if (hiddenGlobally) {
        const value = await read();
        if (current !== generation) return;
        revealed = value;
      }
      visible = true;
    } catch (cause) {
      if (current === generation) showToast(errorMessage(cause), false, entry);
    } finally {
      if (current === generation) busy = false;
    }
  };
  const toggleLabel = $derived(
    busy
      ? t.common_loading_value({ value: t.guard_login_name_label() })
      : visible
        ? t.common_hide_value({ value: t.guard_login_name_label() })
        : t.common_show_value({ value: t.guard_login_name_label() }),
  );
</script>

<span class="fact-row">
  <span class="fact-value">{visible ? revealed || entry.account_name : "******"}</span>
  <button
    class="copy-button"
    type="button"
    title={toggleLabel}
    aria-label={toggleLabel}
    aria-pressed={visible}
    aria-busy={busy}
    disabled={busy}
    onclick={toggle}
  >
    <AsyncIcon icon={visible ? EyeOff : Eye} size={15} {busy} />
  </button>
  <CopyButton
    steamId={entry.steam_id}
    copyKey={`login:${entry.steam_id}`}
    label={t.clipboard_copy_label({ value: t.guard_login_name_label() })}
    valueLabel={t.guard_login_name_label()}
    {read}
    disabled={busy}
  />
</span>
