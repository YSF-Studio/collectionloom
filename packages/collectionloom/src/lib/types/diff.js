/**
 * @typedef {Object} DiffSummary
 * @property {number} total_added
 * @property {number} total_removed
 * @property {number} total_changed
 * @property {number} [high_priority_changes]
 */

/**
 * @typedef {Object} DiffResult
 * @property {string} schema_version
 * @property {string} snapshot_a_id
 * @property {string} snapshot_b_id
 * @property {string} compared_at
 * @property {Record<string, { added: object[], removed: object[], changed: object[] }>} domains
 * @property {DiffSummary} [summary]
 */

export {};
