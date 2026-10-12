import { ErrorApiMessage } from "../errors/errors.constants";
import { t } from "../i18n/i18n.svelte";
import { GuardAction, SteamGuardScreen, SteamGuardField } from "./steam-guard.constants";
import { BackupMode, PickFileKind } from "../vault/backup.constants";
import { untrack } from "svelte";
import { accounts, app, openAccount, showToast, vault, vaultReady } from "../state/state.svelte";
import { invoke } from "../api/tauri";
import { createGuardSession } from "./steam-guard-session.svelte";
import { createLoginController } from "./steam-guard-login.svelte";
import { createLinkController } from "./steam-guard-link.svelte";
import type { GuardFlowType } from "./steam-guard-flow";
import type { SnapshotType } from "../types";

export const createAddAccountController = () => {
  const session = createGuardSession({ needsCode: () => loginFlow.guardNeedsCode });

  let importPath = $state("");
  let importPasskey = $state("");
  //! Only shown once the backend asked for it: most .maFile exports are plain.
  let passkeyNeeded = $state(false);

  const finish = (steamId: string | null): void => {
    app.addAccountOpen = false;
    reset();
    if (steamId) openAccount(steamId);
  };

  const linkFlow = createLinkController({
    session,
    onLinked: (steamId) => finish(steamId),
    onRevoked: () => {},
    onCancelled: () => {},
  });

  const loginFlow = createLoginController({
    session,
    knownAccountName: () => "",
    expectedSteamId: () => null,
    localCodeSteamId: () => null,
    onSignedIn: async (result) => {
      await linkFlow.start(result.steam_id);
    },
    resetAll: () => reset(),
  });

  const reset = (): void => {
    session.reset();
    importPath = "";
    importPasskey = "";
    passkeyNeeded = false;
    loginFlow.reset();
    linkFlow.reset();
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
    app.addAccountOpen = false;
    reset();
  };

  $effect(() => {
    const open = app.addAccountOpen;
    untrack(() => {
      if (open) reset();
    });
  });

  const beginImport = (): void => {
    session.setScreen(SteamGuardScreen.IMPORT);
  };

  const back = (): void => {
    if (session.busy && !loginFlow.waiting) return;
    if (session.screen === SteamGuardScreen.LINK) {
      void linkFlow.cancelLink();
    } else if (session.screen === SteamGuardScreen.LOGIN) {
      void loginFlow.cancelLogin();
    } else {
      reset();
    }
  };

  const importBackup = (): void => {
    app.backup = { mode: BackupMode.IMPORT, steamId: null };
  };

  const pickFile = async (): Promise<void> => {
    const chosen = await session.guard(
      () => invoke("steam_guard_pick_file", { kind: PickFileKind.MAFILE }),
      GuardAction.BROWSE,
    );
    if (typeof chosen === "string") {
      importPath = chosen;
    }
  };

  const newAccountId = (before: Set<string>, snapshot: SnapshotType): string | null => {
    const added = snapshot.accounts.filter((entry) => !before.has(entry.steam_id));
    return added.length === 1 ? added[0].steam_id : null;
  };

  const runImport = async (): Promise<void> => {
    if (!importPath) {
      session.fail(ErrorApiMessage.BACKUP_FILE_REQUIRED, SteamGuardField.IMPORT_FILE);
      return;
    }
    const before = new Set(accounts().map((entry) => entry.steam_id));
    const result = await session.guard(
      () => invoke("steam_guard_import", { path: importPath, passkey: importPasskey || null }),
      GuardAction.IMPORT,
    );
    if (result) {
      session.applyResult(result);
      showToast(t.guard_import_success(), true);
      finish(newAccountId(before, result));
      return;
    }
    if (
      session.errorCode === ErrorApiMessage.BACKUP_PASSWORD_REQUIRED ||
      session.errorCode === ErrorApiMessage.BACKUP_DECRYPTION_FAILED
    ) {
      passkeyNeeded = true;
    }
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
    localCodeSteamId: null,
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
    get vault() {
      return vault();
    },
    get vaultReady() {
      return vaultReady();
    },
    get screen() {
      return session.screen;
    },
    set screen(value: SteamGuardScreen) {
      session.screen = value;
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
    get importPath() {
      return importPath;
    },
    get importPasskey() {
      return importPasskey;
    },
    set importPasskey(value: string) {
      importPasskey = value;
      session.clearFieldError(SteamGuardField.IMPORT_PASSWORD);
    },
    get passkeyNeeded() {
      return passkeyNeeded;
    },
    close,
    back,
    errorFor: session.errorFor,
    beginLink: loginFlow.beginLink,
    beginImport,
    importBackup,
    pickFile,
    runImport,
  };
};

export type AddAccountControllerType = ReturnType<typeof createAddAccountController>;
