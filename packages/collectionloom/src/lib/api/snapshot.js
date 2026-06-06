/** @typedef {import('../types/snapshot.js').SnapshotMeta} SnapshotMeta */
/** @typedef {import('../types/snapshot.js').SnapshotProgress} SnapshotProgress */

import { invoke } from "./tauri.js";

/** @param {string} caseId @param {string} profile */
export async function startSnapshot(caseId, profile) {
  return /** @type {SnapshotMeta} */ (await invoke("start_snapshot", { caseId, profile }));
}

/** @param {string} caseId */
export async function listSnapshots(caseId) {
  return /** @type {SnapshotMeta[]} */ (await invoke("list_snapshots_cmd", { caseId }));
}

/** @param {string} caseId @param {string} snapshotId */
export async function getSnapshot(caseId, snapshotId) {
  return /** @type {SnapshotMeta} */ (await invoke("get_snapshot", { caseId, snapshotId }));
}

/** @param {string} snapshotId */
export async function getSnapshotProgress(snapshotId) {
  return /** @type {SnapshotProgress} */ (await invoke("get_snapshot_progress", { snapshotId }));
}
