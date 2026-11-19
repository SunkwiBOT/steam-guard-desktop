<script lang="ts">
  import { AlertTriangle } from "@lucide/svelte";
  import MessageText from "./MessageText.svelte";
  import type { MessageType } from "../../lib/i18n/messages";

  type FormErrorPropsType = { message: MessageType; id?: string };
  let { message, id }: FormErrorPropsType = $props();
</script>

{#if message}
  <p {id} class="steamguard-error form-error" role="alert">
    <AlertTriangle size={15} />
    <span><MessageText content={message} /></span>
  </p>
{/if}
