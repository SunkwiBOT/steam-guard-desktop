import { untrack } from "svelte";
import { resolveErrorCode } from "../errors/errors.svelte";
import type { ErrorApiMessage } from "../errors/errors.constants";
import { app, vault } from "../state/state.svelte";
import { invoke } from "../api/tauri";
import type { CodeInfoType } from "../types";

export type LiveCodeType = {
  code: string;
  period: number;
  remaining: number;
  error: ErrorApiMessage | "";
};

type CachedCodeType = {
  info: CodeInfoType;
  expiresAt: number;
};

const RETRY_DELAY_MS = 5000;

const cache = $state<Record<string, CachedCodeType>>({});
const errors = $state<Record<string, ErrorApiMessage>>({});
const clock = $state({ now: Date.now() });
const watchers = $state<Record<string, number>>({});
const inFlight = new Set<string>();
const retryAt = new Map<string, number>();
//! Bumped whenever cached codes become stale (lock, accounts changed) so a
//! fetch started before cannot repopulate the cache afterwards.
let generation = 0;

const remainingOf = (entry: CachedCodeType, now: number): number =>
  Math.max(0, Math.ceil((entry.expiresAt - now) / 1000));

const clear = (): void => {
  generation += 1;
  for (const key of Object.keys(cache)) delete cache[key];
  for (const key of Object.keys(errors)) delete errors[key];
  retryAt.clear();
};

export const invalidateCodes = (): void => {
  clear();
  clock.now = Date.now();
};

const fetchCode = async (steamId: string): Promise<void> => {
  if (inFlight.has(steamId)) return;
  const current = generation;
  inFlight.add(steamId);
  try {
    const info = await invoke("steam_guard_code", { steamId });
    if (current !== generation) return;
    cache[steamId] = { info, expiresAt: Date.now() + info.seconds_remaining * 1000 };
    delete errors[steamId];
  } catch (cause) {
    if (current !== generation) return;
    errors[steamId] = resolveErrorCode(cause);
    retryAt.set(steamId, Date.now() + RETRY_DELAY_MS);
  } finally {
    inFlight.delete(steamId);
  }
};

const tick = (): void => {
  const now = Date.now();
  clock.now = now;
  for (const steamId of Object.keys(watchers)) {
    const entry = cache[steamId];
    if (entry && remainingOf(entry, now) > 0) continue;
    if ((retryAt.get(steamId) ?? 0) > now) continue;
    void fetchCode(steamId);
  }
};

/** Registers interest in a live code; the returned function releases it.
 *  ! Untracked: a caller's effect must not depend on the cache it fills. */
export const watchCode = (steamId: string): (() => void) => {
  untrack(() => {
    watchers[steamId] = (watchers[steamId] ?? 0) + 1;
    tick();
  });
  return () => {
    untrack(() => {
      const count = (watchers[steamId] ?? 1) - 1;
      if (count <= 0) delete watchers[steamId];
      else watchers[steamId] = count;
    });
  };
};

export const liveCode = (steamId: string): LiveCodeType | null => {
  const entry = cache[steamId];
  if (!entry) {
    const error = errors[steamId];
    return error ? { code: "", period: 30, remaining: 0, error } : null;
  }
  return {
    code: entry.info.code,
    period: entry.info.period || 30,
    remaining: remainingOf(entry, clock.now),
    error: "",
  };
};

/** One shared 1 s ticker for every visible code; idle while the vault is
 *  locked, the window inactive or nothing is watching. */
export const startCodeTicker = (): (() => void) =>
  $effect.root(() => {
    $effect(() => {
      if (vault().locked || vault().error) untrack(clear);
    });
    $effect(() => {
      const active =
        Object.keys(watchers).length > 0 && !vault().locked && !vault().error && !app.idle;
      if (!active) return;
      untrack(tick);
      const timer = setInterval(tick, 1000);
      return () => clearInterval(timer);
    });
  });
