import {
  checkForUpdateIfDue,
  markDownloaded,
  recordUpdateProgress,
} from "../actions/actions.svelte";
import { t } from "../i18n/i18n.svelte";
import { app, applySnapshot, loadDefaultSettings, run } from "../state/state.svelte";
import { ToastKind } from "../state/state.constants";
import { toast } from "../state/toasts";
import { invalidateCodes } from "../steam-guard/codes.svelte";
import { createVaultOfferGate } from "../vault/vault-offer";
import { invoke, listen } from "./tauri";

const vaultOfferGate = createVaultOfferGate();

const CHECK_INTERVAL_MS = 60 * 1000;

const formatOffset = (seconds: number): string => {
  const sign = seconds < 0 ? "-" : "+";
  const total = Math.abs(seconds);
  const minutes = Math.floor(total / 60);
  const remainder = total % 60;
  return minutes > 0 ? `${sign}${minutes}m ${remainder}s` : `${sign}${remainder}s`;
};

const syncSnapshot = async (): Promise<void> => {
  applySnapshot(await invoke("snapshot"));
};

const setActive = async (active: boolean): Promise<void> => {
  if (app.idle === !active) {
    return;
  }
  app.idle = !active;
  if (active) {
    try {
      await syncSnapshot();
    } catch (e: unknown) {
      void invoke("log_frontend_error", { context: "setActive", message: String(e) }).catch(
        () => {},
      );
    }
    await checkForUpdateIfDue();
  }
};

export const startBridge = async (): Promise<void> => {
  try {
    app.info = await invoke("app_info");
  } catch (e: unknown) {
    void invoke("log_frontend_error", { context: "app_info", message: String(e) }).catch(() => {});
  }
  try {
    await loadDefaultSettings();
  } catch (e: unknown) {
    void invoke("log_frontend_error", { context: "loadDefaultSettings", message: String(e) }).catch(
      () => {},
    );
  }

  await run("snapshot");

  if (app.snapshot && vaultOfferGate.check(app.snapshot.vault, app.snapshot.settings)) {
    app.vaultOfferOpen = true;
  }

  await listen("accounts-changed", async () => {
    invalidateCodes();
    await syncSnapshot();
  });
  await listen("vault-locked", syncSnapshot);
  await listen("update-progress", (event) => recordUpdateProgress(event.payload));
  await listen("update-downloaded", (event) => markDownloaded(event.payload));
  await listen("clock-offset-measured", () => invalidateCodes());
  await listen("clock-drift-warning", (event) => {
    toast(ToastKind.ERROR, t.clock_drift_title(), {
      description: t.clock_drift_warning({ offset: formatOffset(event.payload) }),
    });
  });
  await invoke("frontend_ready");

  window.addEventListener("focus", () => setActive(true));
  window.addEventListener("blur", () => setActive(false));
  document.addEventListener("visibilitychange", () =>
    setActive(document.visibilityState === "visible"),
  );

  void checkForUpdateIfDue(true);
  setInterval(checkForUpdateIfDue, CHECK_INTERVAL_MS);
};
