import type { Component } from "svelte";
import { toast as sonner } from "svelte-sonner";

import type { MessageType } from "../i18n/messages";
import type { ToastAccountType, ToastFolderType } from "../types";
import { TOAST_DURATION_MS, type ToastKind } from "./state.constants";

export type ToastCardPropsType = {
  kind: ToastKind;
  title: string;
  description: MessageType | null;
  account: ToastAccountType | null;
  folder: ToastFolderType | null;
  onclose: () => void;
};

type ToastOptionsType = {
  description?: MessageType;
  account?: ToastAccountType | null;
  folder?: ToastFolderType | null;
  key?: string;
};

let toastId = 0;

let card: Component<ToastCardPropsType>;

export const setToastCard = (component: Component<ToastCardPropsType>): void => {
  card = component;
};

export const dismissToast = (id: string): void => {
  sonner.dismiss(id);
};

export const dismissToasts = (): void => {
  sonner.dismiss();
};

export const toast = (kind: ToastKind, title: string, options: ToastOptionsType = {}): void => {
  toastId += 1;

  const id = options.key ?? `toast-${toastId}`;

  sonner.custom(card, {
    id,
    duration: TOAST_DURATION_MS[kind],
    componentProps: {
      kind,
      title,
      description: options.description ?? null,
      account: options.account ?? null,
      folder: options.folder ?? null,
      onclose: () => dismissToast(id),
    },
  });
};
