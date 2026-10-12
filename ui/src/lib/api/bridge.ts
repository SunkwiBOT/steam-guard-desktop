import {
  checkForUpdateIfDue,
  markDownloaded,
  recordUpdateProgress,
} from "../actions/actions.svelte";
import { app, applySnapshot, run } from "../state/state.svelte";
import { invalidateCodes } from "../steam-guard/codes.svelte";
import { invoke, listen } from "./tauri";

const CHECK_INTERVAL_MS = 60 * 1000;

const syncSnapshot = async (): Promise<void> => {
  applySnapshot(await invoke("snapshot"));
};

const setActive = async (active: boolean): Promise<void> => {
  if (app.idle === !active) {
    return;
  }
  app.idle = !active;
  try {
    await invoke("set_window_active", { active });
  } catch {
    // Best effort: the Rust side also follows the window focus events.
  }
  if (active) {
    // The vault may have locked itself in the background. A failed refresh keeps
    // the last snapshot; the next event or command retries, so never reject here.
    try {
      await syncSnapshot();
    } catch {}
    await checkForUpdateIfDue();
  }
};

export const startBridge = async (): Promise<void> => {
  try {
    app.info = await invoke("app_info");
  } catch {}

  await run("snapshot");

  await listen("accounts-changed", async () => {
    invalidateCodes();
    await syncSnapshot();
  });
  await listen("vault-locked", syncSnapshot);
  await listen("update-progress", (event) => recordUpdateProgress(event.payload ?? {}));
  await listen("update-downloaded", (event) => markDownloaded(String(event.payload ?? "")));

  window.addEventListener("focus", () => setActive(true));
  window.addEventListener("blur", () => setActive(false));
  document.addEventListener("visibilitychange", () =>
    setActive(document.visibilityState === "visible"),
  );

  void checkForUpdateIfDue(true);
  setInterval(checkForUpdateIfDue, CHECK_INTERVAL_MS);
};
