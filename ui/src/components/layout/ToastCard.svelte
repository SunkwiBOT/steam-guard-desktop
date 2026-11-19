<script lang="ts">
  import { CircleCheck, Info, OctagonX, X } from "@lucide/svelte";

  import { t } from "../../lib/i18n/i18n.svelte";
  import { ToastKind } from "../../lib/state/state.constants";
  import type { ToastCardPropsType } from "../../lib/state/toasts";
  import MessageText from "../ui/MessageText.svelte";

  let { kind, title, description, account, folder, onclose }: ToastCardPropsType = $props();
</script>

<div
  class="toast-card"
  class:ok={kind !== ToastKind.ERROR}
  role={kind === ToastKind.ERROR ? "alert" : "status"}
>
  {#if kind === ToastKind.ERROR}
    <OctagonX size={16} class="toast-icon" />
  {:else if kind === ToastKind.SUCCESS}
    <CircleCheck size={16} class="toast-icon" />
  {:else}
    <Info size={16} class="toast-icon" />
  {/if}
  <div class="toast-body">
    <p class="toast-title">{title}</p>
    {#if description || account || folder}
      <p class="toast-description">
        <MessageText content={description} {account} {folder} />
      </p>
    {/if}
  </div>
  <button class="toast-close" type="button" onclick={onclose} aria-label={t.common_close()}>
    <X size={14} />
  </button>
</div>
