/** @typedef {import('../types/diff.js').DiffResult} DiffResult */

import { invoke } from "./tauri.js";

/** @param {string} caseId @param {string} snapshotAId @param {string} snapshotBId */
export async function compareSnapshots(caseId, snapshotAId, snapshotBId) {
  return /** @type {DiffResult} */ (
    await invoke("compare_snapshots", { caseId, snapshotAId, snapshotBId })
  );
}

/** @param {string} caseId */
export async function listDiffs(caseId) {
  return /** @type {DiffResult[]} */ (await invoke("list_diffs_cmd", { caseId }));
}
