/** Toast / status message prefixes (no emoji). */

/** @param {string} m */
export function ok(m) {
  return `OK: ${m}`;
}

/** @param {string} m */
export function err(m) {
  return `ERR: ${m}`;
}

/** @param {string} m */
export function warn(m) {
  return `WARN: ${m}`;
}

/** @param {string} msg */
export function isError(msg) {
  return msg.startsWith("ERR:");
}

/** @param {string} msg */
export function isWarn(msg) {
  return msg.startsWith("WARN:");
}
