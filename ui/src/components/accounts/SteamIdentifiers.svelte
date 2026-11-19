<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { untrack } from "svelte";
  import { Popover } from "bits-ui";
  import { ChevronDown, Eye, EyeOff, Fingerprint } from "@lucide/svelte";
  import { app } from "../../lib/state/state.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import { steamIdentifiers } from "../../lib/accounts/steam-ids";

  type SteamIdentifiersPropsType = { steamId: string };

  let { steamId }: SteamIdentifiersPropsType = $props();
  const rows = $derived(steamIdentifiers(steamId));
  const globallyVisible = $derived(app.snapshot?.settings.show_steam_ids === true);
  let visible = $state(false);
  let previousId = "";
  let previousGlobal: boolean | undefined;
  $effect(() => {
    const id = steamId;
    const show = globallyVisible;
    untrack(() => {
      if (id === previousId && show === previousGlobal) return;
      previousId = id;
      previousGlobal = show;
      visible = show;
    });
  });
</script>

{#if rows.length}
  <Popover.Root>
    <Popover.Trigger class="account-identifiers-trigger">
      <Fingerprint size={15} />
      {t.steam_ids_title()}
      <ChevronDown size={15} class="data-open:rotate-180" />
    </Popover.Trigger>
    <Popover.Portal>
      <Popover.Content sideOffset={6} align="start" class="identifiers-popover">
        <div class="identifiers-toolbar">
          <span>{t.steam_ids_description()}</span>
          <button
            class="copy-button"
            type="button"
            title={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
            aria-label={visible ? t.settings_privacy_hide_ids() : t.steam_ids_show()}
            aria-pressed={visible}
            onclick={() => (visible = !visible)}
          >
            {#if visible}<EyeOff size={15} />{:else}<Eye size={15} />{/if}
          </button>
        </div>
        <dl class="identifiers-list">
          {#each rows as row (row.value)}
            <div>
              <dt>{row.label}</dt>
              <dd>
                <span>{visible ? row.value : "******"}</span>
                <CopyButton
                  {steamId}
                  copyKey={`id:${steamId}:${row.value}`}
                  label={t.clipboard_copy_label({ value: row.label })}
                  valueLabel={row.label}
                  read={() => row.value}
                />
              </dd>
            </div>
          {/each}
        </dl>
      </Popover.Content>
    </Popover.Portal>
  </Popover.Root>
{/if}
