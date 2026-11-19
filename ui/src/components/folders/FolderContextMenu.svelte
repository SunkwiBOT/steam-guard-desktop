<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";

  import ColorSwatches from "./ColorSwatches.svelte";
  import FolderCaption from "./FolderCaption.svelte";
  import FolderIconPicker from "./FolderIconPicker.svelte";
  import { deleteFolder } from "../../lib/actions/actions.svelte";
  import { openFolderEditor } from "../../lib/state/dialogs.svelte";
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuTrigger,
  } from "$lib/components/ui/context-menu/index.js";
  import { app, run } from "../../lib/state/state.svelte";
  import { ArrowRight, SquarePen, Trash2 } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderContextMenuPropsType = {
    folder: FolderDto;
    children: Snippet<[{ props: Record<string, unknown> }]>;
  };

  let { folder, children }: FolderContextMenuPropsType = $props();
  let open = $state(false);

  // ! Dialog-opening actions (edit, delete confirm) prevent the menu's own
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

  const pickSwatch = <T,>(action: (id: string, value: T) => unknown, value: T): void => {
    open = false;
    pick(action, folder.id, value);
  };
</script>

<ContextMenu bind:open>
  <ContextMenuTrigger>
    {#snippet child({ props })}
      {@render children({ props })}
    {/snippet}
  </ContextMenuTrigger>
  <ContextMenuContent {onCloseAutoFocus}>
    <p class="text-muted-foreground px-2 py-1.5 text-sm font-semibold">
      <FolderCaption {folder} size={18} inline bold />
    </p>
    <ContextMenuItem onSelect={() => pick((id: string) => (app.folderId = id), folder.id)}>
      <ArrowRight size={16} />
      {t.common_open()}
    </ContextMenuItem>
    <ContextMenuItem onSelect={() => pick(openFolderEditor, folder.id, undefined, true)}>
      <SquarePen size={16} />
      {t.folder_edit_menu()}
    </ContextMenuItem>
    <ContextMenuSeparator />
    <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
      {t.folder_colour()}
    </p>
    <ColorSwatches
      className="menu-swatches"
      value={folder.color}
      asMenuItems
      onselect={(color) =>
        pickSwatch(
          (id, value: string) => run("set_folder_color", { id, color: value || null }),
          color,
        )}
    />
    <p class="text-muted-foreground px-2 py-1.5 text-xs font-semibold tracking-wide uppercase">
      {t.folder_icon()}
    </p>
    <FolderIconPicker
      className="menu-icons"
      value={folder.icon}
      color={folder.color}
      asMenuItems
      onselect={(icon) =>
        pickSwatch((id, value: string) => run("set_folder_icon", { id, icon: value }), icon)}
    />
    <ContextMenuSeparator />
    <ContextMenuItem
      variant="destructive"
      onSelect={() => pick(deleteFolder, folder.id, undefined, true)}
    >
      <Trash2 size={16} />
      {t.folder_delete_action()}
    </ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
