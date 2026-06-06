import { invoke } from "./tauri.js";

/** @param {string} caseId */
export async function openInAnalysisloom(caseId) {
  return /** @type {string} */ (await invoke("open_in_analysisloom", { caseId }));
}
