import { UiErrorCode } from "../errors/errors.constants";
import { t, rich } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import type { GuardSessionType } from "./steam-guard-session.svelte";
import { askConfirm } from "../state/dialogs.svelte";
import { accountById } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { toast } from "../state/toasts";
import { invoke } from "../api/tauri";
import type { LinkStart } from "../bindings/LinkStart";
import type { SnapshotDto } from "../bindings/SnapshotDto";

export type LinkDepsType = {
  session: GuardSessionType;
  onLinked: (steamId: string, snapshot: SnapshotDto) => void;
  onRevoked: () => void;
  onCancelled: () => void;
};

export const createLinkController = (deps: LinkDepsType) => {
  const { session } = deps;

  let link = $state<LinkStart | null>(null);
  let linkCode = $state("");
  let transferStarted = $state(false);
  let revocationInput = $state("");

  const displayName = (current: LinkStart): string =>
    accountById(current.steam_id)?.display_name || current.account_name || current.steam_id;

  const reset = (): void => {
    link = null;
    linkCode = "";
    transferStarted = false;
    revocationInput = "";
  };

  const start = async (steamId: string): Promise<void> => {
    const started = await session.guard(
      () => invoke("steam_guard_link_start", { steamId }),
      GuardAction.LINK,
    );
    if (started) {
      link = started;
      session.setScreen(SteamGuardScreen.LINK);
      linkCode = "";
      transferStarted = false;
      revocationInput = "";
    }
  };

  const finalizeLink = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const result = await session.guard(
      () =>
        invoke("steam_guard_link_finalize", {
          sessionId: current.session_id,
          confirmCode: linkCode,
        }),
      GuardAction.FINALIZE,
    );
    if (result) {
      session.applyResult(result);
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
      toast(ToastKind.SUCCESS, t.guard_link_success(), { account: accountById(current.steam_id) });
      deps.onLinked(current.steam_id, result);
    }
  };

  const startTransfer = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const done = await session.guardVoid(
      () => invoke("steam_guard_link_transfer_start", { sessionId: current.session_id }),
      GuardAction.TRANSFER_START,
    );
    if (done !== null) {
      transferStarted = true;
    }
  };

  const finishTransfer = async (): Promise<void> => {
    const current = link;
    if (!current) {
      return;
    }
    const result = await session.guard(
      () =>
        invoke("steam_guard_link_transfer_finish", {
          sessionId: current.session_id,
          smsCode: linkCode,
        }),
      GuardAction.TRANSFER_FINISH,
    );
    if (result) {
      session.applyResult(result);
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
      toast(ToastKind.SUCCESS, t.guard_transfer_success(), {
        account: accountById(current.steam_id),
      });
      deps.onLinked(current.steam_id, result);
    }
  };

  const revokeDuringLink = (): void => {
    const current = link;
    if (!current || !revocationInput) {
      session.fail(UiErrorCode.REVOCATION_CODE_REQUIRED, SteamGuardField.REVOCATION);
      return;
    }
    const revocationCode = revocationInput;
    askConfirm({
      title: t.guard_revoke_title(),
      steamId: current.steam_id,
      text: rich(t.guard_revoke_description, { account: displayName(current) }),
      confirmLabel: t.guard_revoke_confirm_action(),
      danger: true,
      action: async () => {
        const done = await session.guardVoid(
          () =>
            invoke("steam_guard_link_revoke", {
              sessionId: current.session_id,
              revocationCode,
            }),
          GuardAction.REVOKE,
        );
        if (done === null) return false;
        link = null;
        session.setScreen(SteamGuardScreen.HOME);
        toast(ToastKind.SUCCESS, t.guard_revoke_success(), {
          account: accountById(current.steam_id),
        });
        deps.onRevoked();
        return true;
      },
    });
  };

  const cancelLink = async (): Promise<void> => {
    if (!link) return;
    const sessionId = link.session_id;
    const done = await session.guardVoid(
      () => invoke("steam_guard_link_cancel", { sessionId }),
      GuardAction.CANCEL_LINK,
    );
    if (done) {
      link = null;
      session.setScreen(SteamGuardScreen.HOME);
      deps.onCancelled();
    }
  };

  const clearRevocation = (): void => {
    revocationInput = "";
  };

  return {
    reset,
    start,
    finalizeLink,
    startTransfer,
    finishTransfer,
    revokeDuringLink,
    cancelLink,
    clearRevocation,
    get link() {
      return link;
    },
    get linkCode() {
      return linkCode;
    },
    set linkCode(value: string) {
      linkCode = value;
      session.clearFieldError(SteamGuardField.LINK_CODE);
    },
    get transferStarted() {
      return transferStarted;
    },
    get revocationInput() {
      return revocationInput;
    },
    set revocationInput(value: string) {
      revocationInput = value;
      session.clearFieldError(SteamGuardField.REVOCATION);
    },
  };
};

export type LinkControllerType = ReturnType<typeof createLinkController>;
