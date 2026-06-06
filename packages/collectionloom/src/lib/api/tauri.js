/** Shared Tauri bridge — safe in browser preview (vite preview / dev without backend). */

import { fixtureInvoke, isFixtureMode } from "./fixtures.js";

/**
 * Rust IPC types (generated from ysf-core via ts-rs).
 * Regenerate: `npm run generate:types`
 *
 * @typedef {import('./generated/DiskInfo').DiskInfo} DiskInfo
 * @typedef {import('./generated/DiskPartitionInfo').DiskPartitionInfo} DiskPartitionInfo
 * @typedef {import('./generated/ProgressState').ProgressState} ProgressState
 * @typedef {import('./generated/ImagingSummary').ImagingSummary} ImagingSummary
 * @typedef {import('./generated/WriteBlockerStatus').WriteBlockerStatus} WriteBlockerStatus
 * @typedef {import('./generated/HpaDcoReport').HpaDcoReport} HpaDcoReport
 * @typedef {import('./generated/PreflightReport').PreflightReport} PreflightReport
 * @typedef {import('./generated/PreflightCheck').PreflightCheck} PreflightCheck
 * @typedef {import('./generated/PortableLayout').PortableLayout} PortableLayout
 * @typedef {import('./generated/StorageCheckReport').StorageCheckReport} StorageCheckReport
 * @typedef {import('./generated/EvidenceHashReport').EvidenceHashReport} EvidenceHashReport
 * @typedef {import('./generated/MobileDevice').MobileDevice} MobileDevice
 * @typedef {import('./generated/ProcessEntry').ProcessEntry} ProcessEntry
 * @typedef {import('./generated/EncryptionReport').EncryptionReport} EncryptionReport
 */

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
  try {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return await tauriInvoke(cmd, args);
  } catch (err) {
    console.error(`[CollectionLoom IPC] invoke('${cmd}') failed:`, err);
    throw err;
  }
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
