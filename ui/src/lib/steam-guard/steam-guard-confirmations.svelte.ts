import type { UiErrorCode } from "../errors/errors.constants";
import { errorMessage, resolveErrorCode } from "../errors/errors.svelte";
import { rich, t } from "../i18n/i18n.svelte";
import type { GuardBusyActionType } from "./steam-guard.constants";
import { isCurrentResponse, requiresExtraConfirmation } from "./steam-guard-rules";
import { ToastKind } from "../state/state.constants";
import { toast } from "../state/toasts";
import { askConfirm, confirmDialog } from "../state/dialogs.svelte";
import type { ToastAccountType } from "../types";
import { invoke } from "../api/tauri";
import type { ConfirmationItem } from "../bindings/ConfirmationItem";
import type { ConfirmationKind as ConfirmationKindDto } from "../bindings/ConfirmationKind";

const KIND_LABELS: Record<ConfirmationKindDto, () => string> = {
  TRADE: () => t.guard_confirmation_kind_trade(),
  MARKET: () => t.guard_confirmation_kind_market(),
  PHONE: () => t.guard_confirmation_kind_phone(),
  RECOVERY: () => t.guard_confirmation_kind_recovery(),
  API_KEY: () => t.guard_confirmation_kind_api_key(),
  FEATURE: () => t.guard_confirmation_kind_opt_out(),
  FAMILY: () => t.guard_confirmation_kind_family(),
  TEST: () => t.guard_confirmation_kind_test(),
  OTHER: () => t.guard_confirmation_kind_unknown(),
};

export const confirmationKindLabel = (kind: ConfirmationKindDto): string => KIND_LABELS[kind]();

export type ConfirmationsDepsType = {
  steamId: () => string | null;
  account: () => ToastAccountType | null;
  generation: () => number;
  isBusy: () => boolean;
  runVoid: (action: () => Promise<void>, name: GuardBusyActionType) => Promise<boolean | null>;
};

export const createConfirmationsController = (deps: ConfirmationsDepsType) => {
  let items = $state<ConfirmationItem[]>([]);
  let busy = $state(false);
  let loaded = $state(false);
  let attempted = $state(false);
  let error = $state<UiErrorCode | "">("");
  let revision = 0;
  let refreshQueued = false;

  const isCurrent = (requested: { generation: number; revision: number }): boolean =>
    isCurrentResponse(requested, { generation: deps.generation(), revision });

  const invalidate = (): void => {
    revision += 1;
  };

  const clear = (): void => {
    invalidate();
    items = [];
    loaded = false;
    attempted = false;
    error = "";
    refreshQueued = false;
  };

  const reset = (): void => {
    clear();
    busy = false;
  };

  const load = async (): Promise<void> => {
    const steamId = deps.steamId();
    if (!steamId || deps.isBusy()) return;
    if (busy) {
      refreshQueued = true;
      return;
    }
    const requested = { generation: deps.generation(), revision };
    busy = true;
    attempted = true;
    error = "";
    try {
      const list = await invoke("steam_guard_confirmations", { steamId });
      if (isCurrent(requested)) {
        items = list;
        loaded = true;
      }
    } catch (e: unknown) {
      if (isCurrent(requested)) error = resolveErrorCode(e);
    } finally {
      if (requested.generation === deps.generation()) {
        busy = false;
        if (refreshQueued) {
          refreshQueued = false;
          void load();
        }
      }
    }
  };

  const act = (item: ConfirmationItem, accept: boolean): void => {
    const steamId = deps.steamId();
    if (!steamId || deps.isBusy()) return;

    const perform = async (): Promise<boolean> => {
      invalidate();
      const done = await deps.runVoid(
        () =>
          invoke("steam_guard_confirmation_action", {
            steamId,
            id: item.id,
            nonce: item.nonce,
            accept,
          }),
        `confirmation:${item.id}:${accept}`,
      );
      if (done === null) {
        await load();
        return false;
      }
      items = items.filter((candidate) => candidate.id !== item.id);
      toast(
        ToastKind.SUCCESS,
        accept ? t.guard_confirmation_accept_success() : t.guard_confirmation_refuse_success(),
        { account: deps.account() },
      );
      await load();
      return true;
    };

    if (accept && requiresExtraConfirmation(item.kind)) {
      if (confirmDialog.open) return;
      const detail = item.summary
        ? `${item.headline} — ${item.summary.replace(/\n/g, " · ")}`
        : item.headline;
      askConfirm({
        title: t.guard_confirmation_extra_title(),
        text: rich(t.guard_confirmation_extra_description, {
          kind: confirmationKindLabel(item.kind),
          detail,
        }),
        confirmLabel: t.guard_confirmation_extra_accept(),
        danger: true,
        action: perform,
      });
      return;
    }

    void perform();
  };

  return {
    get items() {
      return items;
    },
    get busy() {
      return busy;
    },
    get loaded() {
      return loaded;
    },
    get attempted() {
      return attempted;
    },
    get error() {
      return error ? errorMessage(error) : "";
    },
    invalidate,
    clear,
    reset,
    load,
    act,
  };
};
