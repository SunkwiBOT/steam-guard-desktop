<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";
  import { app } from "../../lib/state/state.svelte";
  import { invoke } from "../../lib/api/tauri";
  import SecretReveal from "../ui/SecretReveal.svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  type AccountIdentityPropsType = { entry: AccountDto };

  let { entry }: AccountIdentityPropsType = $props();
  const needsFetch = $derived(app.snapshot?.masked !== false || !entry.account_name);

  const read = (): Promise<string> =>
    needsFetch
      ? invoke("account_login_name", { steamId: entry.steam_id })
      : Promise.resolve(entry.account_name);
</script>

<SecretReveal
  {entry}
  resetKey={`${entry.steam_id}:${needsFetch}`}
  mask="******"
  defaultVisible={!needsFetch}
  {needsFetch}
  fallbackValue={entry.account_name}
  copyKey={`login:${entry.steam_id}`}
  copyLabel={t.clipboard_copy_label({ value: t.guard_login_name_label() })}
  copyValueLabel={t.guard_login_name_label()}
  {read}
  toggleLabel={({ busy, visible }) =>
    busy
      ? t.common_loading_value({ value: t.guard_login_name_label() })
      : visible
        ? t.common_hide_value({ value: t.guard_login_name_label() })
        : t.common_show_value({ value: t.guard_login_name_label() })}
/>
