<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import {
    accounts,
    app,
    emptyMessage,
    visibleAccounts,
    visibleFolders,
  } from "../../lib/state/state.svelte";
  import AccountCard from "../accounts/AccountCard.svelte";
  import FolderCard from "../folders/FolderCard.svelte";
  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";
  import { ArrowLeft, Plus } from "@lucide/svelte";

  const entries = $derived(visibleAccounts());
  const folders = $derived(visibleFolders());
  const insideFolder = $derived(app.folderId !== null && !app.search.trim());
  const nothingSaved = $derived(app.snapshot !== null && accounts().length === 0);
  const empty = $derived(entries.length === 0 && folders.length === 0 && !insideFolder);
</script>

{#if !empty}
  <div class="cards">
    {#each folders as folder (folder.id)}
      <FolderCard {folder} />
    {/each}

    {#if insideFolder}
      <button
        type="button"
        class="card back-card"
        title={t.folder_leave()}
        onclick={() => (app.folderId = null)}
      >
        <span class="folder-icon" aria-hidden="true"><ArrowLeft size={24} /></span>
        <span class="card-name">{t.common_back()}</span>
      </button>
    {/if}

    {#each entries as entry (entry.steam_id)}
      <AccountCard {entry} />
    {/each}
  </div>
{:else if nothingSaved}
  <div class="empty-state welcome">
    <span class="empty-icon" aria-hidden="true"><SteamGuardIcon size={30} /></span>
    <h2 class="empty-title">{t.accounts_empty_title()}</h2>
    <p>{t.accounts_empty()}</p>
    <button class="button primary" type="button" onclick={() => (app.addAccountOpen = true)}
      ><Plus size={15} /> {t.account_add_title()}</button
    >
  </div>
{:else if app.snapshot}
  <p class="empty-state">{emptyMessage()}</p>
{/if}
