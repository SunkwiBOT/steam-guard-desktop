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
  <div
    class="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] auto-rows-fr items-start gap-2.5 pb-2"
  >
    {#each folders as folder (folder.id)}
      <FolderCard {folder} />
    {/each}

    {#if insideFolder}
      <button
        type="button"
        class="card back-card group relative flex h-full min-h-[168px] flex-col items-center justify-center gap-1.5 overflow-hidden rounded-md border border-[var(--border)] bg-[var(--base-alt)] px-2.5 pt-2.5 pb-2 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:border-[var(--border-strong)] hover:bg-[var(--surface-hover)] focus-visible:shadow-[inset_0_0_0_2px_var(--accent)] focus-visible:outline-none"
        title={t.folder_leave()}
        onclick={() => (app.folderId = null)}
      >
        <span
          class="mt-1 flex size-[62px] shrink-0 items-center justify-center rounded-full bg-[var(--avatar)] text-[var(--text)]"
          aria-hidden="true"><ArrowLeft size={24} /></span
        >
        <span
          class="w-full overflow-hidden text-ellipsis whitespace-nowrap text-center font-semibold text-[var(--text-strong)]"
          >{t.common_back()}</span
        >
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
