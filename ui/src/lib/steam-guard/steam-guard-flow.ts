import type {
  GuardBusyActionType,
  GuardKind,
  LoginPurpose,
  SteamGuardField,
} from "./steam-guard.constants";
import type { LinkStartType, LoginBeginType } from "../types";

/** The part of a controller the sign-in and link screens render: shared by
 *  the account dialog and the add-account dialog. */
export type GuardFlowType = {
  readonly busy: boolean;
  readonly busyAction: GuardBusyActionType;
  readonly waiting: boolean;
  readonly loginBegin: LoginBeginType | null;
  readonly selectedGuard: GuardKind | null;
  readonly guardNeedsCode: boolean;
  readonly loginPurpose: LoginPurpose;
  readonly accountNameLocked: boolean;
  accountName: string;
  password: string;
  loginCode: string;
  readonly link: LinkStartType | null;
  linkCode: string;
  readonly transferStarted: boolean;
  revocationInput: string;
  /** Account whose saved authenticator can supply the device code, if any. */
  readonly localCodeSteamId: string | null;
  errorFor: (field: SteamGuardField) => string;
  startPassword: () => Promise<void>;
  startQr: () => Promise<void>;
  needsExternalConfirmation: () => boolean;
  cancelLogin: () => Promise<void>;
  submitPasswordCode: () => Promise<void>;
  finalizeLink: () => Promise<void>;
  startTransfer: () => Promise<void>;
  finishTransfer: () => Promise<void>;
  revokeDuringLink: () => Promise<void>;
  cancelLink: () => Promise<void>;
};
