<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";
  import { invoke } from "../../lib/api/tauri";
  import SecretReveal from "../ui/SecretReveal.svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  type RevocationCodePropsType = { entry: AccountDto };

  let { entry }: RevocationCodePropsType = $props();

  const read = (): Promise<string> =>
    invoke("steam_guard_revocation_code", { steamId: entry.steam_id });
</script>

<SecretReveal
  {entry}
  resetKey={entry.steam_id}
  mask="••••••"
  accentWhenVisible
  copyKey={`revocation:${entry.steam_id}`}
  copyLabel={t.clipboard_copy_label({ value: t.guard_revocation_value() })}
  copyValueLabel={t.guard_revocation_label()}
  {read}
  toggleLabel={({ visible }) =>
    visible
      ? t.common_hide_value({ value: t.guard_revocation_value() })
      : t.common_show_value({ value: t.guard_revocation_value() })}
/>
