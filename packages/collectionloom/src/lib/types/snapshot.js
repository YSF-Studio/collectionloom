/**
 * @typedef {Object} ModuleResult
 * @property {string} name
 * @property {string} status
 * @property {number} [duration_ms]
 * @property {string} [error]
 * @property {number} [items_count]
 */

/**
 * @typedef {Object} SnapshotMeta
 * @property {string} schema_version
 * @property {string} snapshot_id
 * @property {string} case_id
 * @property {{ hostname: string }} host
 * @property {{ family: string, version: string }} os
 * @property {string} profile
 * @property {string} started_at
 * @property {string} completed_at
 * @property {number} [duration_seconds]
 * @property {'completed'|'partial'|'failed'} status
 * @property {ModuleResult[]} [modules]
 * @property {string} integrity_hash
 */

/**
 * @typedef {Object} SnapshotProgress
 * @property {string} snapshot_id
 * @property {boolean} running
 * @property {string|null} current_module
 * @property {number} percent
 */

export {};
