/** @typedef {import('../types/case.js').Case} Case */

import { invoke } from "./tauri.js";

/** @param {{ title: string, operator: string, purpose?: string, timezone: string, description?: string }} input */
export async function createCase(input) {
  return /** @type {Case} */ (await invoke("create_case", input));
}

/** @param {{ status?: string, search?: string }} [filter] */
export async function listCases(filter = {}) {
  return /** @type {Case[]} */ (await invoke("list_cases_cmd", filter));
}

/** @returns {Promise<Array<{ case: Case, snapshot_count: number, export_count: number, diff_count: number, case_dir: string }>>} */
export async function listCaseSummaries() {
  return invoke("list_case_summaries_cmd");
}

/** @param {string} caseId */
export async function getCase(caseId) {
  return /** @type {Case} */ (await invoke("get_case", { caseId }));
}
