/** Shared Tauri bridge — safe in browser preview (vite preview / dev without backend). */

import { fixtureInvoke, isFixtureMode } from "./fixtures.js";

export const PREVIEW_MODE = "PREVIEW_MODE";

/** @returns {boolean} */
export function isTauri() {
  if (typeof window === "undefined") return false;
  return !!(window.__TAURI_INTERNALS__ ?? window.__TAURI__);
}

/** @param {unknown} err */
export function isPreviewError(err) {
  return err instanceof Error && err.message === PREVIEW_MODE;
}

export async function invoke(cmd, args = {}) {
  if (isFixtureMode()) {
    return fixtureInvoke(cmd, args);
  }
  if (!isTauri()) {
    throw new Error(PREVIEW_MODE);
  }
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke(cmd, args);
}

/** @param {import('@tauri-apps/plugin-dialog').OpenDialogOptions} [options] */
export async function openDialog(options) {
  if (!isTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  return open(options);
}

/** @param {string} path */
export async function openPath(path) {
  if (!isTauri()) return;
  const { open } = await import("@tauri-apps/plugin-shell");
  return open(path);
}

/** @param {string} event @param {(payload: import('@tauri-apps/api/event').Event<any>) => void} handler */
export async function listenEvent(event, handler) {
  if (!isTauri()) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen(event, handler);
}
