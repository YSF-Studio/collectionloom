/** Human-readable write-blocker pill label. */
export function wbPillLabel(wbStatus) {
  if (!wbStatus) return "Inactive";
  const active = !!(wbStatus.active ?? wbStatus.enabled);
  if (!active) return "Inactive";
  const type = wbStatus.hardware ? "hardware" : wbStatus.software ? "software" : wbStatus.method || "";
  return type ? `Active (${type})` : "Active";
}
