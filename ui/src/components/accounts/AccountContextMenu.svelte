<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "../folders/ColorSwatches.svelte";
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
  } from "$lib/components/ui/context-menu/index.js";
  import {
    copySteamGuardCode,
    createFolderFor,
    exportAccount,
    forgetAccount,
    moveToFolder,
    refreshProfile,
    setAccountColor,
    signInAgain,
  } from "../../lib/actions/actions.svelte";
  import { folderById, folders, openAccount } from "../../lib/state/state.svelte";
  import { Copy, Download, FolderPlus, LogIn, RefreshCw, Trash2 } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  import SteamGuardIcon from "../steam-guard/SteamGuardIcon.svelte";

  type AccountContextMenuPropsType = {
    entry: AccountDto;
    children: Snippet<[{ props: Record<string, unknown> }]>;
  };

  let { entry, children }: AccountContextMenuPropsType = $props();
  let open = $state(false);

  const folderName = $derived(
    (entry.folder_id && folderById(entry.folder_id)?.name) || t.folder_none(),
  );

  // ! Dialog-opening actions (forget, export, create-folder) prevent the menu's own
  // close-focus restoration so it doesn't steal focus back from the dialog that action
  // opens; bits-ui runs that restoration via `Content`'s `onCloseAutoFocus`.
  let suppressCloseFocus = false;

  const pick = <T,>(
    action: (id: string, value: T) => unknown,
    id: string,
    value?: T,
    opensDialog = false,
  ): void => {
    if (!id) return;
    suppressCloseFocus = opensDialog;
    void action(id, value as T);
  };

  const onCloseAutoFocus = (event: Event): void => {
    if (!suppressCloseFocus) return;
    event.preventDefault();
    suppressCloseFocus = false;
  };

  const pickSwatch = (action: (id: string, value: string) => unknown, color: string): void => {
    open = false;
    pick(action, entry.steam_id, color);
  };
</script>

<ContextMenu bind:open>
  <ContextMenuTrigger>
    {#snippet child({ props })}
      {@render children({ props })}
    {/snippet}
  </ContextMenuTrigger>
  <ContextMenuContent {onCloseAutoFocus}>
    <ContextMenuItem onSelect={() => pick(openAccount, entry.steam_id)}>
      <SteamGuardIcon size={16} />
      {t.guard_open_action()}
    </ContextMenuItem>
    {#if !entry.pending}
      <ContextMenuItem variant="accent" onSelect={() => pick(copySteamGuardCode, entry.steam_id)}>
        <Copy size={16} />
        {t.guard_code_copy()}
      </ContextMenuItem>
    {/if}
    {#if !entry.has_tokens && !entry.pending}
      <ContextMenuItem onSelect={() => pick(signInAgain, entry.steam_id)}>
        <LogIn size={16} />
        {t.guard_sign_in_again()}
      </ContextMenuItem>
    {/if}
    <ContextMenuSeparator />
    <ContextMenuSub>
      <ContextMenuSubTrigger>
        <FolderPlus size={15} />
        {t.folder_move_label()}
        <span class="ml-auto text-xs text-muted-foreground">{folderName}</span>
      </ContextMenuSubTrigger>
      <ContextMenuSubContent>
        <ContextMenuRadioGroup
          value={entry.folder_id ?? ""}
          onValueChange={(value) => pick(moveToFolder, entry.steam_id, value || null)}
        >
          <ContextMenuRadioItem value="">{t.folder_none()}</ContextMenuRadioItem>
          {#each folders() as folder (folder.id)}
            <ContextMenuRadioItem value={folder.id}>{folder.name}</ContextMenuRadioItem>
          {/each}
        </ContextMenuRadioGroup>
      </ContextMenuSubContent>
    </ContextMenuSub>
    <ContextMenuItem onSelect={() => pick(createFolderFor, entry.steam_id, undefined, true)}>
      <FolderPlus size={16} />
      {t.folder_create_menu()}
    </ContextMenuItem>
    <ContextMenuSeparator />
    <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
      {t.account_colour()}
    </p>
    <ColorSwatches
      className="menu-swatches"
      value={entry.color}
      asMenuItems
      onselect={(color) => pickSwatch(setAccountColor, color)}
    />
    <ContextMenuSeparator />
    <ContextMenuItem onSelect={() => pick(refreshProfile, entry.steam_id)}>
      <RefreshCw size={16} />
      {t.profile_refresh_action()}
    </ContextMenuItem>
    {#if !entry.pending}
      <ContextMenuItem onSelect={() => pick(exportAccount, entry.steam_id, undefined, true)}>
        <Download size={16} />
        {t.backup_export_account_action()}
      </ContextMenuItem>
    {/if}
    <ContextMenuItem
      variant="destructive"
      onSelect={() => pick(forgetAccount, entry.steam_id, undefined, true)}
    >
      <Trash2 size={16} />
      {t.guard_forget_action()}
    </ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
