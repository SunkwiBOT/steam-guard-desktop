<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import FolderCaption from "../folders/FolderCaption.svelte";
  import FolderIconPicker from "../folders/FolderIconPicker.svelte";
  import { nextMenuIndex } from "../../lib/state/keyboard";
  import {
    copySteamGuardCode,
    createFolderFor,
    deleteFolder,
    exportAccount,
    forgetAccount,
    moveToFolder,
    refreshProfile,
    setAccountColor,
    signInAgain,
  } from "../../lib/actions/actions.svelte";
  import { openFolderEditor } from "../../lib/state/dialogs.svelte";
  import {
    closeContextMenu,
    contextMenu,
    dropdown,
    openDropdown,
  } from "../../lib/state/menus.svelte";
  import {
    accountById,
    app,
    folderById,
    folders,
    openAccount,
    run,
  } from "../../lib/state/state.svelte";
  import {
    ArrowRight,
    ChevronDown,
    Copy,
    Download,
    FolderPlus,
    LogIn,
    RefreshCw,
    SquarePen,
    Trash2,
  } from "@lucide/svelte";

  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";

  let element = $state<HTMLDivElement | null>(null);
  let position = $state({ left: 0, top: 0 });
  let folderButton = $state<HTMLButtonElement | null>(null);

  const entry = $derived(contextMenu.steamId ? accountById(contextMenu.steamId) : null);
  const folder = $derived(contextMenu.folderId ? folderById(contextMenu.folderId) : null);
  const folderName = $derived(
    (entry?.folder_id && folderById(entry.folder_id)?.name) || t.folder_none(),
  );

  $effect(() => {
    if (!contextMenu.open || !element) {
      return;
    }
    const width = element.offsetWidth;
    const height = element.offsetHeight;
    position = {
      left: Math.max(8, Math.min(contextMenu.x, window.innerWidth - width - 8)),
      top: Math.max(8, Math.min(contextMenu.y, window.innerHeight - height - 8)),
    };
  });

  $effect(() => {
    if (!contextMenu.open || !element) {
      return;
    }
    element.querySelector<HTMLElement>("[role='menuitem']")?.focus();
  });

  const moveFocus = (event: KeyboardEvent): void => {
    if (!element) return;
    const items = [...element.querySelectorAll<HTMLElement>("[role='menuitem']")];
    const next = nextMenuIndex(
      items.findIndex((item) => item === document.activeElement),
      items.length,
      event.key,
    );
    if (next === null) return;
    event.preventDefault();
    items[next]?.focus();
  };

  /** Closes the menu first: the action may open a dialog that takes the focus. */
  const pick = <T,>(action: (id: string, value: T) => unknown, id: string, value?: T): void => {
    closeContextMenu();
    if (id) void action(id, value as T);
  };

  const pickFolder = (): void => {
    if (!entry || !folderButton) return;
    const account = entry;
    openDropdown({
      anchor: folderButton,
      value: account.folder_id ?? "",
      options: [
        { value: "", label: t.folder_none() },
        ...folders().map((item) => ({ value: item.id, label: item.name })),
      ],
      onSelect: (value) => pick(moveToFolder, account.steam_id, value || null),
    });
  };
</script>

{#if contextMenu.open && (entry || folder)}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    bind:this={element}
    style="left:{position.left}px;top:{position.top}px"
    onkeydown={moveFocus}
  >
    {#if entry}
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => pick(openAccount, entry.steam_id)}
      >
        <span class="menu-icon"><SteamGuardIcon size={16} /></span>
        {t.guard_open_action()}
      </button>
      {#if !entry.pending}
        <button
          role="menuitem"
          class="menu-item accent"
          type="button"
          onclick={() => pick(copySteamGuardCode, entry.steam_id)}
        >
          <span class="menu-icon"><Copy size={16} /></span>
          {t.guard_code_copy()}
        </button>
      {/if}
      {#if !entry.has_tokens && !entry.pending}
        <button
          role="menuitem"
          class="menu-item"
          type="button"
          onclick={() => pick(signInAgain, entry.steam_id)}
        >
          <span class="menu-icon"><LogIn size={16} /></span>
          {t.guard_sign_in_again()}
        </button>
      {/if}
      <hr />
      <p class="menu-heading">{t.folder_move_label()}</p>
      <div class="menu-folder-picker">
        <button
          class="select-button"
          type="button"
          aria-label={t.folder_move_label()}
          aria-haspopup="listbox"
          aria-expanded={dropdown.open && dropdown.anchor === folderButton}
          bind:this={folderButton}
          onclick={pickFolder}
        >
          <span>{folderName}</span>
          <span class="picker-caret"><ChevronDown size={14} /></span>
        </button>
      </div>
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => pick(createFolderFor, entry.steam_id)}
      >
        <span class="menu-icon"><FolderPlus size={16} /></span>
        {t.folder_create_menu()}
      </button>
      <hr />
      <p class="menu-heading">{t.account_colour()}</p>
      <ColorSwatches
        className="menu-swatches"
        value={entry.color}
        onselect={(color) => pick(setAccountColor, entry.steam_id, color)}
      />
      <hr />
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => pick(refreshProfile, entry.steam_id)}
      >
        <span class="menu-icon"><RefreshCw size={16} /></span>
        {t.profile_refresh_action()}
      </button>
      {#if !entry.pending}
        <button
          role="menuitem"
          class="menu-item"
          type="button"
          onclick={() => pick(exportAccount, entry.steam_id)}
        >
          <span class="menu-icon"><Download size={16} /></span>
          {t.backup_export_account_action()}
        </button>
      {/if}
      <button
        role="menuitem"
        class="menu-item danger"
        type="button"
        onclick={() => pick(forgetAccount, entry.steam_id)}
      >
        <span class="menu-icon"><Trash2 size={16} /></span>
        {t.guard_forget_action()}
      </button>
    {:else if folder}
      <p class="menu-heading folder-heading">
        <FolderCaption {folder} size={18} inline bold />
      </p>
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => pick((id) => (app.folderId = id), folder.id)}
      >
        <span class="menu-icon"><ArrowRight size={16} /></span>
        {t.common_open()}
      </button>
      <button
        role="menuitem"
        class="menu-item"
        type="button"
        onclick={() => pick(openFolderEditor, folder.id)}
      >
        <span class="menu-icon"><SquarePen size={16} /></span>
        {t.folder_edit_menu()}
      </button>
      <hr />
      <p class="menu-heading">{t.folder_colour()}</p>
      <ColorSwatches
        className="menu-swatches"
        value={folder.color}
        onselect={(color) =>
          pick(
            (id, value: string) => run("set_folder_color", { id, color: value || null }),
            folder.id,
            color,
          )}
      />
      <p class="menu-heading">{t.folder_icon()}</p>
      <FolderIconPicker
        className="menu-icons"
        value={folder.icon}
        color={folder.color}
        onselect={(icon) =>
          pick((id, value: string) => run("set_folder_icon", { id, icon: value }), folder.id, icon)}
      />
      <hr />
      <button
        role="menuitem"
        class="menu-item danger"
        type="button"
        onclick={() => pick(deleteFolder, folder.id)}
      >
        <span class="menu-icon"><Trash2 size={16} /></span>
        {t.folder_delete_action()}
      </button>
    {/if}
  </div>
{/if}
