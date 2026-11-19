<script lang="ts">
  import DialogHeader from "../dialogs/DialogHeader.svelte";
  import AppDialog from "../dialogs/AppDialog.svelte";
  import { DialogContent } from "$lib/components/ui/dialog/index.js";
  import ColorSwatches from "./ColorSwatches.svelte";
  import FolderIconPicker from "./FolderIconPicker.svelte";
  import UnsavedBar from "../ui/UnsavedBar.svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import { folderIconId } from "../../lib/folders/folder-style";
  import { folderDraftDirty } from "../../lib/folders/folder-editor";
  import { app } from "../../lib/state/state.svelte";
  import { SquarePen } from "@lucide/svelte";
  import { untrack } from "svelte";
  import type { FolderDto } from "../../lib/bindings/FolderDto";

  type FolderEditorValueType = { name: string; color: string | null; icon: string | null };
  type FolderEditorPropsType = {
    folder: FolderDto;
    onsave: (value: FolderEditorValueType) => Promise<boolean>;
    onclose: () => void;
  };

  let { folder, onsave, onclose }: FolderEditorPropsType = $props();
  const baseline = untrack(() => ({
    name: folder.name,
    color: folder.color,
    icon: folderIconId(folder.icon, app.info.folder_icons),
  }));
  let name = $state(baseline.name);
  let color = $state(baseline.color ?? "");
  let icon = $state(baseline.icon);
  let saving = $state(false);
  const dirty = $derived(folderDraftDirty({ name, color, icon }, baseline));
  const uid = $props.id();

  const close = (): void => {
    if (saving) return;
    onclose();
  };

  const save = async (): Promise<void> => {
    if (saving || !name.trim()) return;
    saving = true;
    try {
      if (await onsave({ name: name.trim(), color: color || null, icon })) onclose();
    } finally {
      saving = false;
    }
  };
</script>

<AppDialog open={true} onclose={close}>
  <DialogContent showCloseButton={false} class="sm:max-w-md" aria-labelledby={`${uid}-title`}>
    <DialogHeader
      id={`${uid}-title`}
      title={t.folder_edit_title()}
      icon={SquarePen}
      onclose={close}
      closeDisabled={saving}
    />
    <div class="folder-editor-name">
      <label class="field-label" for={`${uid}-name`}>{t.folder_name()}</label>
      <input
        id={`${uid}-name`}
        class="input full"
        type="text"
        spellcheck="false"
        bind:value={name}
        disabled={saving}
        onkeydown={(event) => event.key === "Enter" && (event.preventDefault(), void save())}
      />
    </div>
    <div class="folder-editor-fields">
      <div class="folder-editor-field">
        <span class="field-label">{t.folder_colour()}</span>
        <ColorSwatches value={color} disabled={saving} onselect={(value) => (color = value)} />
      </div>
      <div class="folder-editor-field">
        <span class="field-label">{t.folder_icon()}</span>
        <FolderIconPicker
          value={icon}
          {color}
          disabled={saving}
          onselect={(value) => (icon = value)}
        />
      </div>
    </div>
    {#if dirty}
      <UnsavedBar {saving} oncancel={close} onsave={save} saveDisabled={!name.trim()} />
    {/if}
  </DialogContent>
</AppDialog>
