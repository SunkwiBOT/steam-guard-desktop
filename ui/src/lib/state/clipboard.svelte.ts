import { errorMessage } from "../errors/errors.svelte";
import { t, rich } from "../i18n/i18n.svelte";
import { ToastKind } from "./state.constants";
import { toast } from "./toasts";
import { invoke } from "../api/tauri";
import type { ToastAccountType } from "../types";

export const copyFeedback = $state<Record<string, boolean>>({});
const timers = new Map<string, ReturnType<typeof setTimeout>>();

export const copyValue = async (
  value: string,
  key: string,
  label: string,
  account: ToastAccountType | null = null,
): Promise<boolean> => {
  try {
    await invoke("clipboard_write_text", { text: value });
    clearTimeout(timers.get(key));
    copyFeedback[key] = true;
    timers.set(
      key,
      setTimeout(() => {
        delete copyFeedback[key];
        timers.delete(key);
      }, 2000),
    );
    toast(ToastKind.SUCCESS, t.clipboard_copied(), {
      description: rich(t.clipboard_copy_success, { value: label }),
      account,
    });
    return true;
  } catch (e: unknown) {
    toast(ToastKind.ERROR, t.action_failed_title(), { description: errorMessage(e), account });
    return false;
  }
};
