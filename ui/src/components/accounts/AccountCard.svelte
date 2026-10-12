<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  import { openAccountMenu } from "../../lib/state/menus.svelte";
  import { app, folderById, openAccount, vault } from "../../lib/state/state.svelte";
  import { liveCode, watchCode } from "../../lib/steam-guard/codes.svelte";
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import Avatar from "../ui/Avatar.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import { Hourglass, Lock, LogIn } from "@lucide/svelte";
  import type { AccountType } from "../../lib/types";

  type AccountCardPropsType = { entry: AccountType };

  let { entry }: AccountCardPropsType = $props();

  const locked = $derived(vault().locked || Boolean(vault().error));
  const showCode = $derived(!locked && !entry.pending);
  const code = $derived(showCode ? liveCode(entry.steam_id) : null);
  const progress = $derived(code && code.code ? (code.remaining / code.period) * 100 : 0);
  //! The root list mixes folder contents while searching, so a card has to say
  //! which folder it comes from; inside a folder every card shares it.
  const folder = $derived(
    app.folderId === null && entry.folder_id ? folderById(entry.folder_id) : null,
  );
  const Glyph = $derived(folderIcon(folder?.icon));
  const folderStyle = $derived(
    folder?.color ? `${tintStyle(folder.color)};border-color:${folder.color}` : "",
  );

  $effect(() => {
    if (!showCode) return;
    return watchCode(entry.steam_id);
  });
</script>

<!-- ! A group rather than a button: it holds buttons of its own. Enter and a
     click open the account; the copy button stops its own click. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="card"
  class:pending={entry.pending}
  class:locked
  class:has-folder={Boolean(folder)}
  role="group"
  tabindex="0"
  aria-label={entry.display_name}
  title={t.account_card_tooltip()}
  onclick={() => openAccount(entry.steam_id)}
  oncontextmenu={(event) => openAccountMenu(event, entry.steam_id)}
  onkeydown={(event) => {
    if (event.target !== event.currentTarget || event.key !== "Enter") return;
    event.preventDefault();
    openAccount(entry.steam_id);
  }}
>
  {#if entry.color}
    <span class="card-accent" style="--card-accent:{entry.color}" aria-hidden="true"></span>
  {/if}
  {#if folder}
    <span class="card-folder" style={folderStyle} title={folder.name} aria-hidden="true">
      <Glyph size={13} />
    </span>
  {/if}
  {#if entry.pending}
    <span class="card-badge">{t.account_pending()}</span>
  {/if}
  {#if locked}
    <span class="card-lock" title={t.account_locked_hint()} aria-hidden="true"
      ><Lock size={14} /></span
    >
  {:else if !entry.pending}
    <CopyButton
      className="card-view"
      steamId={entry.steam_id}
      copyKey={`guard:${entry.steam_id}`}
      label={t.guard_code_copy()}
      title={t.guard_code_copy()}
      guard
      action={() => copySteamGuardCode(entry.steam_id)}
    />
  {/if}

  <Avatar {entry} size={56} extra="card-avatar" />
  <span class="card-name">{entry.display_name}</span>

  <div class="card-code" class:paused={app.idle} class:blurred={app.snapshot?.settings.blur_codes}>
    {#if entry.pending}
      <span class="card-hint"><Hourglass size={12} /> {t.account_pending_hint()}</span>
    {:else if locked}
      <span class="card-hint"><Lock size={12} /> {t.account_locked_hint()}</span>
    {:else if code?.code}
      <span class="card-code-value" aria-label={t.guard_code_label()}>{code.code}</span>
      <div class="card-timer" class:urgent={code.remaining <= 5} aria-hidden="true">
        <span style="width:{progress.toFixed(1)}%"></span>
      </div>
    {:else if code?.error}
      <span class="card-hint error">{errorMessage(code.error)}</span>
    {:else}
      <span class="card-hint">{t.guard_code_loading()}</span>
    {/if}
  </div>
  <!-- ! Always rendered: the slot keeps the code at the same height on every card. -->
  <span
    class="card-meta"
    title={!entry.has_tokens && !entry.pending ? t.account_sign_in_tooltip() : undefined}
  >
    {#if !entry.has_tokens && !entry.pending}<LogIn size={12} /> {t.account_sign_in_hint()}{/if}
  </span>
</div>
