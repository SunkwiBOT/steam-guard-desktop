import { t } from "../i18n/i18n.svelte";
import { app } from "../state/state.svelte";
import { vaultCountdown } from "./vault-lock";

/** Label of the time left before the vault locks itself, or "" when it will
 *  not. Must be created inside a component or an effect root. */
export const createLockCountdown = () => {
  const lockInSeconds = $derived(app.snapshot?.vault.lock_in_seconds ?? null);
  let remaining = $state<number | null>(null);
  //! Anchored on a deadline, not decremented: a throttled background timer
  //! cannot desync it, and regaining focus snaps it back immediately.
  let lockAt = 0;
  $effect(() => {
    const seconds = lockInSeconds;
    if (seconds === null) {
      remaining = null;
      return;
    }
    lockAt = Date.now() + seconds * 1000;
    const sync = (): void => {
      remaining = Math.max(0, Math.ceil((lockAt - Date.now()) / 1000));
    };
    sync();
    const timer = setInterval(sync, 1000);
    window.addEventListener("focus", sync);
    document.addEventListener("visibilitychange", sync);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", sync);
      document.removeEventListener("visibilitychange", sync);
    };
  });
  const label = $derived.by(() => {
    if (remaining === null) return "";
    const parts = vaultCountdown(remaining);
    if ("hours" in parts) {
      return t.vault_lock_remaining_hours({
        hours: parts.hours,
        minutes: parts.minutes,
        seconds: parts.seconds,
      });
    }
    if ("minutes" in parts) {
      return t.vault_lock_remaining_minutes({ minutes: parts.minutes, seconds: parts.seconds });
    }
    return t.vault_lock_remaining_seconds({ seconds: parts.seconds });
  });
  return {
    get label() {
      return label;
    },
  };
};
