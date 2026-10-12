import { t, rich } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, LoginPurpose } from "./steam-guard.constants";
import { SettingsSection } from "../state/state.constants";
import { untrack } from "svelte";
import { accountById, app, showToast, vault, vaultReady } from "../state/state.svelte";
import { invoke } from "../api/tauri";
import { askConfirm } from "../state/dialogs.svelte";
import { exportAccount, forgetAccount, refreshProfile } from "../actions/actions.svelte";
import { createConfirmationsController } from "./steam-guard-confirmations.svelte";
import { createGuardSession } from "./steam-guard-session.svelte";
import { createLoginController } from "./steam-guard-login.svelte";
import { createLinkController } from "./steam-guard-link.svelte";
import type { GuardFlowType } from "./steam-guard-flow";

export const createSteamGuardController = () => {
  const entry = $derived(accountById(app.steamGuardAccountId ?? ""));
  const selectedAccountId = $derived(entry?.steam_id ?? null);

  const session = createGuardSession({ needsCode: () => loginFlow.guardNeedsCode });

  const confirmations = createConfirmationsController({
    steamId: () => entry?.steam_id ?? null,
    account: () => entry,
    generation: () => session.generation(),
    isBusy: () => session.busy,
    runVoid: session.guardVoid,
  });

  const linkFlow = createLinkController({
    session,
    onLinked: () => confirmations.clear(),
    onRevoked: () => {},
    onCancelled: () => {},
  });

  const completeSignIn = async (): Promise<void> => {
    session.setScreen(SteamGuardScreen.HOME);
    confirmations.clear();
    session.applyResult(await invoke("snapshot"));
    showToast(t.guard_sign_in_success(), true, entry);
  };

  const loginFlow = createLoginController({
    session,
    knownAccountName: () => entry?.account_name ?? "",
    expectedSteamId: () => entry?.steam_id ?? null,
    localCodeSteamId: () => (entry && !entry.pending ? entry.steam_id : null),
    onSignedIn: async (result, purpose) => {
      if (purpose === LoginPurpose.LINK) await linkFlow.start(result.steam_id);
      else await completeSignIn();
    },
    resetAll: () => reset(),
  });

  const reset = (): void => {
    session.reset();
    confirmations.reset();
    loginFlow.reset();
    linkFlow.reset();
  };

  const back = (): void => {
    if (session.busy && !loginFlow.waiting) return;
    if (session.screen === SteamGuardScreen.LINK) {
      void linkFlow.cancelLink();
    } else if (session.screen === SteamGuardScreen.LOGIN) {
      void loginFlow.cancelLogin();
    } else {
      session.setScreen(SteamGuardScreen.HOME);
    }
  };

  const close = (): void => {
    if (session.busy && !loginFlow.waiting) return;
    if (loginFlow.loginBegin && !linkFlow.link) {
      void invoke("steam_guard_login_cancel", {
        sessionId: loginFlow.loginBegin.session_id,
      }).catch(() => {});
    }
    if (linkFlow.link) {
      void invoke("steam_guard_link_cancel", { sessionId: linkFlow.link.session_id }).catch(
        () => {},
      );
    }
    app.steamGuardOpen = false;
    app.steamGuardAccountId = null;
    reset();
  };

  $effect(() => {
    const open = app.steamGuardOpen;
    const accountId = selectedAccountId;
    untrack(() => {
      reset();
      if (open && !accountId) app.steamGuardOpen = false;
      // Opened from "sign in again": land on the form, unless the vault must be unlocked first.
      const signIn = app.steamGuardSignIn;
      app.steamGuardSignIn = false;
      if (open && accountId && signIn && vaultReady()) loginFlow.beginSignIn();
    });
  });

  $effect(() => {
    if (
      app.steamGuardOpen &&
      session.screen === SteamGuardScreen.HOME &&
      entry?.has_tokens &&
      !entry.pending &&
      vaultReady() &&
      !app.idle
    ) {
      untrack(() => {
        if (!confirmations.attempted) void confirmations.load();
      });
    }
  });

  const openVaultSettings = (): void => {
    close();
    app.settingsSection = SettingsSection.VAULT;
    app.settingsOpen = true;
  };

  /** A pending link resumes directly when Steam tokens are still held,
   *  otherwise the sign-in comes first. */
  const resumeActivation = async (): Promise<void> => {
    if (!entry) return;
    if (entry.has_tokens) await linkFlow.start(entry.steam_id);
    else loginFlow.beginLink();
  };

  const removeAuthenticator = async (): Promise<void> => {
    if (!entry) {
      return;
    }
    const answer = await askConfirm({
      title: t.guard_remove_title(),
      steamId: entry.steam_id,
      text: rich(t.guard_remove_description, { account: entry.display_name }),
      confirmLabel: t.guard_remove_action(),
      danger: true,
    });
    if (!answer) {
      return;
    }
    const result = await session.guard(
      () => invoke("steam_guard_remove", { steamId: entry.steam_id, revocationCode: null }),
      GuardAction.REMOVE,
    );
    if (result) {
      session.applyResult(result);
      linkFlow.clearRevocation();
      confirmations.clear();
      showToast(t.guard_remove_success(), true, entry);
      close();
    }
  };

  const forget = async (): Promise<void> => {
    if (!entry || session.busy) return;
    if (await forgetAccount(entry.steam_id)) close();
  };

  const refresh = async (): Promise<void> => {
    if (!entry || session.busy) return;
    session.busy = true;
    try {
      await refreshProfile(entry.steam_id);
    } finally {
      session.busy = false;
    }
  };

  const exportThis = (): void => {
    if (entry) exportAccount(entry.steam_id);
  };

  const flow: GuardFlowType = {
    get busy() {
      return session.busy;
    },
    get busyAction() {
      return session.busyAction;
    },
    get waiting() {
      return loginFlow.waiting;
    },
    get loginBegin() {
      return loginFlow.loginBegin;
    },
    get selectedGuard() {
      return loginFlow.selectedGuard;
    },
    get guardNeedsCode() {
      return loginFlow.guardNeedsCode;
    },
    get loginPurpose() {
      return loginFlow.loginPurpose;
    },
    get accountNameLocked() {
      return loginFlow.accountNameLocked;
    },
    get accountName() {
      return loginFlow.accountName;
    },
    set accountName(value: string) {
      loginFlow.accountName = value;
    },
    get password() {
      return loginFlow.password;
    },
    set password(value: string) {
      loginFlow.password = value;
    },
    get loginCode() {
      return loginFlow.loginCode;
    },
    set loginCode(value: string) {
      loginFlow.loginCode = value;
    },
    get link() {
      return linkFlow.link;
    },
    get linkCode() {
      return linkFlow.linkCode;
    },
    set linkCode(value: string) {
      linkFlow.linkCode = value;
    },
    get transferStarted() {
      return linkFlow.transferStarted;
    },
    get revocationInput() {
      return linkFlow.revocationInput;
    },
    set revocationInput(value: string) {
      linkFlow.revocationInput = value;
    },
    get localCodeSteamId() {
      return entry && !entry.pending ? entry.steam_id : null;
    },
    errorFor: session.errorFor,
    startPassword: loginFlow.startPassword,
    startQr: loginFlow.startQr,
    needsExternalConfirmation: loginFlow.needsExternalConfirmation,
    cancelLogin: loginFlow.cancelLogin,
    submitPasswordCode: loginFlow.submitPasswordCode,
    finalizeLink: linkFlow.finalizeLink,
    startTransfer: linkFlow.startTransfer,
    finishTransfer: linkFlow.finishTransfer,
    revokeDuringLink: linkFlow.revokeDuringLink,
    cancelLink: linkFlow.cancelLink,
  };

  return {
    flow,
    get entry() {
      return entry;
    },
    get vault() {
      return vault();
    },
    get vaultReady() {
      return vaultReady();
    },
    get screen() {
      return session.screen;
    },
    get busy() {
      return session.busy;
    },
    set busy(value: boolean) {
      session.busy = value;
    },
    get busyAction() {
      return session.busyAction;
    },
    get waiting() {
      return loginFlow.waiting;
    },
    get link() {
      return linkFlow.link;
    },
    get confirmations() {
      return confirmations.items;
    },
    get confirmationsBusy() {
      return confirmations.busy;
    },
    get confirmationsLoaded() {
      return confirmations.loaded;
    },
    get confirmationsAttempted() {
      return confirmations.attempted;
    },
    get confirmationsError() {
      return confirmations.error;
    },
    close,
    back,
    errorFor: session.errorFor,
    openVaultSettings,
    resumeActivation,
    beginSignIn: loginFlow.beginSignIn,
    loadConfirmations: confirmations.load,
    actOnConfirmation: confirmations.act,
    removeAuthenticator,
    forget,
    refresh,
    exportThis,
  };
};

export type SteamGuardControllerType = ReturnType<typeof createSteamGuardController>;
