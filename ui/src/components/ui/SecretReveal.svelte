<script lang="ts">
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { untrack } from "svelte";
  import { Eye, EyeOff } from "@lucide/svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import { toast } from "../../lib/state/toasts";
  import AsyncIcon from "./AsyncIcon.svelte";
  import CopyButton from "./CopyButton.svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  type SecretRevealStateType = { busy: boolean; visible: boolean };

  type SecretRevealPropsType = {
    entry: AccountDto;
    resetKey: string;
    mask: string;
    copyKey: string;
    copyLabel: string;
    copyValueLabel: string;
    read: () => Promise<string>;
    toggleLabel: (state: SecretRevealStateType) => string;
    defaultVisible?: boolean;
    needsFetch?: boolean;
    fallbackValue?: string;
    accentWhenVisible?: boolean;
  };

  let {
    entry,
    resetKey,
    mask,
    copyKey,
    copyLabel,
    copyValueLabel,
    read,
    toggleLabel,
    defaultVisible = false,
    needsFetch = true,
    fallbackValue = "",
    accentWhenVisible = false,
  }: SecretRevealPropsType = $props();

  let visible = $state(false);
  let revealed = $state("");
  let busy = $state(false);
  let generation = 0;
  let previousKey = "";

  // ! Hidden again whenever the reset key changes: the value is a secret.
  $effect(() => {
    const key = resetKey;
    untrack(() => {
      if (key === previousKey) return;
      previousKey = key;
      generation += 1;
      visible = defaultVisible;
      revealed = "";
      busy = false;
    });
  });

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
      if (needsFetch) {
        const value = await read();
        if (current !== generation) return;
        revealed = value;
      }
      visible = true;
    } catch (e: unknown) {
      if (current === generation)
        toast(ToastKind.ERROR, t.action_failed_title(), {
          description: errorMessage(e),
          account: entry,
        });
    } finally {
      if (current === generation) busy = false;
    }
  };
</script>

<span class="fact-row">
  <span class="fact-value" class:accent={accentWhenVisible && visible}
    >{visible ? revealed || fallbackValue : mask}</span
  >
  <button
    class="copy-button"
    type="button"
    title={toggleLabel({ busy, visible })}
    aria-label={toggleLabel({ busy, visible })}
    aria-pressed={visible}
    aria-busy={busy}
    disabled={busy}
    onclick={toggle}
  >
    <AsyncIcon icon={visible ? EyeOff : Eye} size={15} {busy} />
  </button>
  <CopyButton
    steamId={entry.steam_id}
    {copyKey}
    label={copyLabel}
    valueLabel={copyValueLabel}
    {read}
    disabled={busy}
  />
</span>
