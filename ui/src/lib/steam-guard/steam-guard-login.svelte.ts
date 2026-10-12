import { ErrorApiMessage } from "../errors/errors.constants";
import { resolveErrorCode } from "../errors/errors.svelte";
import {
  GuardAction,
  SteamGuardScreen,
  SteamGuardField,
  LoginPurpose,
  GuardKind,
} from "./steam-guard.constants";
import type { GuardSessionType } from "./steam-guard-session.svelte";
import type { LoginBeginType, LoginResultType } from "../types";
import { invoke } from "../api/tauri";

export type LoginDepsType = {
  session: GuardSessionType;
  /** Sign-in name already known for the account, "" when the user must type it. */
  knownAccountName: () => string;
  /** Steam ID the login must belong to, null for a brand-new account. */
  expectedSteamId: () => string | null;
  /** Account whose saved authenticator can supply the device code, if any. */
  localCodeSteamId: () => string | null;
  onSignedIn: (result: LoginResultType, purpose: LoginPurpose) => Promise<void>;
  resetAll: () => void;
};

export const createLoginController = (deps: LoginDepsType) => {
  const { session } = deps;

  let accountName = $state("");
  let password = $state("");
  let loginBegin = $state<LoginBeginType | null>(null);
  let selectedGuard = $state<GuardKind | null>(null);
  let loginCode = $state("");
  let waiting = $state(false);
  let loginPurpose = $state<LoginPurpose>(LoginPurpose.LINK);

  const guardNeedsCode = $derived(
    selectedGuard === GuardKind.DEVICE_CODE || selectedGuard === GuardKind.EMAIL_CODE,
  );
  const accountNameLocked = $derived(deps.knownAccountName() !== "");

  const reset = (): void => {
    accountName = "";
    password = "";
    loginBegin = null;
    selectedGuard = null;
    loginCode = "";
    waiting = false;
    loginPurpose = LoginPurpose.LINK;
  };

  const cancelLogin = async (): Promise<void> => {
    if (session.busyAction === GuardAction.CANCEL_LOGIN) return;
    const sessionId = loginBegin?.session_id;
    if (!sessionId) {
      deps.resetAll();
      return;
    }
    session.invalidate();
    session.setBusy(false);
    waiting = false;
    const cancelled = await session.guardVoid(
      () => invoke("steam_guard_login_cancel", { sessionId }),
      GuardAction.CANCEL_LOGIN,
    );
    if (cancelled) deps.resetAll();
  };

  const chooseMethod = (methods: LoginBeginType["methods"]): void => {
    const codeMethod = methods.find(
      (method) => method.kind === GuardKind.DEVICE_CODE || method.kind === GuardKind.EMAIL_CODE,
    );
    selectedGuard = codeMethod?.kind ?? methods[0]?.kind ?? null;
  };

  const startPassword = async (): Promise<void> => {
    const name = (deps.knownAccountName() || accountName).trim();
    if (!name) {
      session.fail(ErrorApiMessage.ACCOUNT_NAME_REQUIRED, SteamGuardField.ACCOUNT_NAME);
      return;
    }
    if (!password) {
      session.fail(ErrorApiMessage.STEAM_PASSWORD_REQUIRED, SteamGuardField.PASSWORD);
      return;
    }
    const begin = await session.guard(
      () => invoke("steam_guard_login_password", { accountName: name, password }),
      GuardAction.PASSWORD,
    );
    if (begin) {
      loginBegin = begin;
      password = "";
      chooseMethod(begin.methods);
    }
  };

  const startQr = async (): Promise<void> => {
    const begin = await session.guard(() => invoke("steam_guard_login_qr"), GuardAction.QR);
    if (begin) {
      loginBegin = begin;
      selectedGuard = null;
      waiting = true;
      void finishLogin(null, null);
    }
  };

  const needsExternalConfirmation = (): boolean => {
    return (
      selectedGuard === GuardKind.DEVICE_CONFIRMATION ||
      selectedGuard === GuardKind.EMAIL_CONFIRMATION
    );
  };

  const finishLogin = async (kind: GuardKind | null, value: string | null): Promise<void> => {
    if (!loginBegin) {
      return;
    }
    const sessionId = loginBegin.session_id;
    const current = session.generation();
    waiting = true;
    try {
      const signedIn = await session.guard(
        () => invoke("steam_guard_login_finish", { sessionId, guardKind: kind, code: value }),
        GuardAction.LOGIN,
      );
      if (!signedIn) {
        return;
      }
      const expected = deps.expectedSteamId();
      if (expected && signedIn.steam_id !== expected) {
        session.fail(ErrorApiMessage.ACCOUNT_MISMATCH, SteamGuardField.LOGIN);
        loginBegin = null;
        return;
      }
      loginBegin = null;
      await deps.onSignedIn(signedIn, loginPurpose);
    } catch (cause) {
      if (current === session.generation()) {
        session.fail(resolveErrorCode(cause), SteamGuardField.LOGIN);
      }
    } finally {
      if (current === session.generation()) waiting = false;
    }
  };

  const begin = (purpose: LoginPurpose): void => {
    loginPurpose = purpose;
    loginBegin = null;
    loginCode = "";
    accountName = deps.knownAccountName();
    session.clearError();
    session.setScreen(SteamGuardScreen.LOGIN);
  };

  const beginSignIn = (): void => begin(LoginPurpose.SIGN_IN);

  const beginLink = (): void => begin(LoginPurpose.LINK);

  const submitPasswordCode = async (): Promise<void> => {
    if (guardNeedsCode) {
      let value = loginCode.trim();
      const steamId = deps.localCodeSteamId();
      if (!value && selectedGuard === GuardKind.DEVICE_CODE && steamId) {
        const current = await session.guard(
          () => invoke("steam_guard_code", { steamId }),
          GuardAction.LOGIN_CODE,
        );
        if (!current) return;
        value = current.code;
      }
      if (!value) {
        session.fail(ErrorApiMessage.GUARD_CODE_REQUIRED, SteamGuardField.LOGIN_CODE);
        return;
      }
      await finishLogin(selectedGuard, value);
    } else {
      await finishLogin(null, null);
    }
  };

  return {
    reset,
    cancelLogin,
    startPassword,
    startQr,
    needsExternalConfirmation,
    beginSignIn,
    beginLink,
    submitPasswordCode,
    get guardNeedsCode() {
      return guardNeedsCode;
    },
    get accountNameLocked() {
      return accountNameLocked;
    },
    get accountName() {
      return accountName;
    },
    set accountName(value: string) {
      accountName = value;
      session.clearFieldError(SteamGuardField.ACCOUNT_NAME);
    },
    get password() {
      return password;
    },
    set password(value: string) {
      password = value;
      session.clearFieldError(SteamGuardField.PASSWORD);
    },
    get loginBegin() {
      return loginBegin;
    },
    get selectedGuard() {
      return selectedGuard;
    },
    get loginCode() {
      return loginCode;
    },
    set loginCode(value: string) {
      loginCode = value;
      session.clearFieldError(SteamGuardField.LOGIN_CODE);
    },
    get waiting() {
      return waiting;
    },
    get loginPurpose() {
      return loginPurpose;
    },
  };
};

export type LoginControllerType = ReturnType<typeof createLoginController>;
