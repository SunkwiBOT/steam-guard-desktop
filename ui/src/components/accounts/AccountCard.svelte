<script lang="ts">
  import { t } from "../../lib/i18n/i18n.svelte";
  import { cn } from "$lib/utils.js";

  import AccountContextMenu from "./AccountContextMenu.svelte";
  import { errorMessage } from "../../lib/errors/errors.svelte";
  import { copySteamGuardCode } from "../../lib/actions/actions.svelte";
  import { app, folderById, openAccount, vault } from "../../lib/state/state.svelte";
  import { liveCode, watchCode } from "../../lib/steam-guard/codes.svelte";
  import { folderIcon } from "../../lib/folders/folder-icons";
  import { tintStyle } from "../../lib/folders/folder-style";
  import Avatar from "../ui/Avatar.svelte";
  import CopyButton from "../ui/CopyButton.svelte";
  import MessageText from "../ui/MessageText.svelte";
  import { Hourglass, Lock, LogIn } from "@lucide/svelte";
  import type { AccountDto } from "../../lib/bindings/AccountDto";

  type AccountCardPropsType = { entry: AccountDto };

  let { entry }: AccountCardPropsType = $props();

  const locked = $derived(vault().locked || Boolean(vault().error));
  const showCode = $derived(!locked && !entry.pending);
  const code = $derived(showCode ? liveCode(entry.steam_id) : null);
  const progress = $derived(code && code.code ? (code.remaining / code.period) * 100 : 0);
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

<AccountContextMenu {entry}>
  {#snippet children({ props })}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      {...props}
      class="card group relative flex h-full min-h-[168px] cursor-pointer flex-col items-center gap-1.5 overflow-hidden rounded-md border border-[var(--border)] bg-[var(--base-alt)] px-2.5 pt-2.5 pb-2 text-center text-inherit transition-[transform,border-color,background-color] duration-[120ms] ease-out hover:z-[2] hover:border-[var(--border-strong)] hover:bg-[var(--surface-hover)] focus-visible:shadow-[inset_0_0_0_2px_var(--accent)] focus-visible:outline-none [&_img]:pointer-events-none [&_img]:[-webkit-user-drag:none] [&_img]:select-none [&_svg]:pointer-events-none [&_svg]:select-none"
      role="group"
      tabindex="0"
      aria-label={entry.display_name}
      title={t.account_card_tooltip()}
      onclick={() => openAccount(entry.steam_id)}
      onkeydown={(event) => {
        if (event.target !== event.currentTarget || event.key !== "Enter") return;
        event.preventDefault();
        openAccount(entry.steam_id);
      }}
    >
      {#if entry.color}
        <span
          class="absolute inset-y-0 left-0 w-1"
          style="background:{entry.color}"
          aria-hidden="true"
        ></span>
      {/if}
      {#if folder}
        <span
          class="absolute top-2 left-2 inline-flex size-[22px] items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] text-[var(--accent)]"
          style={folderStyle}
          title={folder.name}
          aria-hidden="true"
        >
          <Glyph size={14} />
        </span>
      {/if}
      {#if entry.pending}
        <span
          class={cn(
            "absolute top-2 rounded-full border border-[var(--warning-border)] bg-[var(--warning-bg)] px-[7px] py-px text-[10px] font-bold uppercase text-[var(--warning-text)]",
            folder ? "left-9" : "left-2",
          )}>{t.account_pending()}</span
        >
      {/if}
      {#if locked}
        <span
          class="absolute top-2 right-2 inline-flex size-[22px] items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] text-[var(--muted)]"
          title={t.account_locked_hint()}
          aria-hidden="true"><Lock size={14} /></span
        >
      {:else if !entry.pending}
        <CopyButton
          className="absolute top-2 right-2 inline-flex size-[25px] cursor-copy items-center justify-center rounded-full border border-[var(--border-strong)] bg-[var(--panel)] p-0 text-[var(--text)] opacity-0 transition-opacity duration-[120ms] group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 not-disabled:hover:border-[var(--accent)] not-disabled:hover:bg-[var(--button)] not-disabled:hover:text-[var(--text-bright)] disabled:cursor-not-allowed aria-busy:disabled:cursor-progress disabled:opacity-45"
          steamId={entry.steam_id}
          copyKey={`guard:${entry.steam_id}`}
          label={t.guard_code_copy()}
          title={t.guard_code_copy()}
          guard
          action={() => copySteamGuardCode(entry.steam_id)}
        />
      {/if}

      <Avatar {entry} size={56} extra={cn("mt-1 text-[22px]", locked && "opacity-70")} />
      <span
        class="w-full overflow-hidden text-ellipsis whitespace-nowrap text-center font-semibold text-[var(--text-strong)]"
        >{entry.display_name}</span
      >

      <div class="mt-auto flex min-h-10 w-full flex-col items-center justify-end gap-1.5">
        {#if entry.pending}
          <span class="inline-flex min-h-[22px] items-center gap-1 text-[11px] text-[var(--muted)]"
            ><Hourglass size={12} /> {t.account_pending_hint()}</span
          >
        {:else if locked}
          <span class="inline-flex min-h-[22px] items-center gap-1 text-[11px] text-[var(--muted)]"
            ><Lock size={12} /> {t.account_locked_hint()}</span
          >
        {:else if code?.code}
          <span
            class={cn(
              "-mr-1 font-mono text-[22px] leading-none font-bold tracking-[4px] text-[var(--text-strong)]",
              (app.idle || app.snapshot?.settings.blur_codes) && "blur-[5px]",
            )}
            aria-label={t.guard_code_label()}>{code.code}</span
          >
          <div class="h-1 w-full overflow-hidden rounded-sm bg-[var(--grid)]" aria-hidden="true">
            <span
              class={cn(
                "block h-full transition-[width] duration-1000 ease-linear",
                code.remaining <= 5 ? "bg-[var(--warning-text)]" : "bg-[var(--accent)]",
              )}
              style="width:{progress.toFixed(1)}%"
            ></span>
          </div>
        {:else if code?.error}
          <span
            class="inline-flex min-h-[22px] items-center gap-1 text-[11px] text-[var(--error-text)]"
            ><MessageText content={errorMessage(code.error)} /></span
          >
        {:else}
          <span class="inline-flex min-h-[22px] items-center gap-1 text-[11px] text-[var(--muted)]"
            >{t.guard_code_loading()}</span
          >
        {/if}
      </div>
      <span
        class="inline-flex min-h-5 max-w-full items-center gap-1 text-[11px] whitespace-nowrap text-[var(--caution-text)] [&>svg]:shrink-0"
        title={!entry.has_tokens && !entry.pending ? t.account_sign_in_tooltip() : undefined}
      >
        {#if !entry.has_tokens && !entry.pending}<LogIn size={12} />
          {t.account_sign_in_hint()}{/if}
      </span>
    </div>
  {/snippet}
</AccountContextMenu>
