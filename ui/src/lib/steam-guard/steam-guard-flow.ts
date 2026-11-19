import type {
  GuardBusyActionType,
  GuardKind,
  LoginPurpose,
  SteamGuardField,
} from "./steam-guard.constants";
import type { LinkStart } from "../bindings/LinkStart";
import type { LoginBegin } from "../bindings/LoginBegin";
import type { MessageType } from "../i18n/messages";

export type GuardFlowType = {
  readonly busy: boolean;
  readonly busyAction: GuardBusyActionType;
  readonly waiting: boolean;
  readonly loginBegin: LoginBegin | null;
  readonly selectedGuard: GuardKind | null;
  readonly guardNeedsCode: boolean;
  readonly loginPurpose: LoginPurpose;
  readonly accountNameLocked: boolean;
  accountName: string;
  password: string;
  loginCode: string;
  readonly link: LinkStart | null;
  linkCode: string;
  readonly transferStarted: boolean;
  revocationInput: string;
  readonly localCodeSteamId: string | null;
  errorFor: (field: SteamGuardField) => MessageType;
  startPassword: () => Promise<void>;
  startQr: () => Promise<void>;
  needsExternalConfirmation: () => boolean;
  cancelLogin: () => Promise<void>;
  submitPasswordCode: () => Promise<void>;
  finalizeLink: () => Promise<void>;
  startTransfer: () => Promise<void>;
  finishTransfer: () => Promise<void>;
  revokeDuringLink: () => void;
  cancelLink: () => Promise<void>;
};
