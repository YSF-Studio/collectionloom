/** @typedef {import('../types/export.js').ExportResult} ExportResult */

import { invoke } from "./tauri.js";

/** @param {string} caseId @param {string} snapshotId */
export async function exportJson(caseId, snapshotId) {
  return /** @type {ExportResult} */ (await invoke("export_json", { caseId, snapshotId }));
}

/** @param {string} caseId @param {string} snapshotId @param {boolean} includeDiff */
export async function exportMarkdown(caseId, snapshotId, includeDiff = false) {
  return /** @type {ExportResult} */ (
    await invoke("export_markdown", { caseId, snapshotId, includeDiff })
  );
}

/** @param {string} caseId */
export async function exportZip(caseId) {
  return /** @type {ExportResult} */ (await invoke("export_zip", { caseId }));
}

/** @param {string} caseId */
export async function listExports(caseId) {
  return /** @type {ExportResult[]} */ (await invoke("list_exports", { caseId }));
}
