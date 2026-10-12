<script lang="ts">
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import type { ToastFolderType } from "../../lib/types";

  type FolderCaptionPropsType = {
    folder: ToastFolderType;
    size?: number;
    inline?: boolean;
    bold?: boolean;
  };

  let { folder, size = 28, inline = false, bold = false }: FolderCaptionPropsType = $props();
  const Glyph = $derived(folderIcon(folder.icon));
  const glyphSize = $derived(Math.round(size * 0.6));
  const badgeStyle = $derived(
    [`--folder-size:${size}px`, tintStyle(folder.color)].filter(Boolean).join(";"),
  );
</script>

<span class="account-caption folder-caption" class:inline
  ><span class="folder-badge" style={badgeStyle} aria-hidden="true"><Glyph size={glyphSize} /></span
  ><span title={folder.name}
    >{#if bold}<strong>{folder.name}</strong>{:else}{folder.name}{/if}</span
  ></span
>
