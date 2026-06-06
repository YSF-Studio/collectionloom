/**
 * @typedef {Object} CaseOperator
 * @property {string} name
 * @property {string} [badge_id]
 * @property {string} [email]
 */

/**
 * @typedef {Object} Case
 * @property {string} schema_version
 * @property {string} case_id
 * @property {string} title
 * @property {string} [description]
 * @property {CaseOperator} operator
 * @property {string} [purpose]
 * @property {string} timezone
 * @property {string} created_at
 * @property {string} [updated_at]
 * @property {'open'|'in_review'|'closed'|'archived'} status
 * @property {string[]} [tags]
 * @property {string} [notes]
 */

export {};
