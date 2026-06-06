<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import { writeBlockerGuide } from "../guides.js";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();
let device = $state("");
let status = $state("Unknown");
let enabled = $state(false);
let blockerMethod = $state("");

// Detect platform and method
let platform = $state("");
$effect(() => {
  if (typeof navigator !== "undefined" && navigator.platform) {
    platform = navigator.platform;
  }
  // Determine method based on platform
  if (platform.startsWith("Linux")) blockerMethod = "mount rdonly / BLKROSET";
  else if (platform.startsWith("Win")) blockerMethod = "DeviceIoControl";
  else if (platform.startsWith("Mac")) blockerMethod = "mount rdonly";
  else blockerMethod = "Unknown";
});

let blockerNotes = $state("");

async function checkBlockerStatus() {
  try {
    const r = await timeoutPromise(invoke("check_write_blocker", { device }), 5000);
    if (r && typeof r === "object") {
      enabled = !!(r.active ?? r.enabled);
      status = enabled ? "Enabled" : "Disabled";
      if (r.method) blockerMethod = r.method;
      if (r.notes) blockerNotes = r.notes;
    } else if (typeof r === "boolean") {
      enabled = r;
      status = r ? "Enabled" : "Disabled";
    }
  } catch {
    /* non-fatal */
  }
}

async function enable() {
  setBusy(true);
  try {
    await timeoutPromise(invoke("enable_write_blocker", { device }), 10000);
    await checkBlockerStatus();
    setMsg("OK: Write blocker enabled");
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    setMsg(`ERR: ${err}`);
  }
  setBusy(false);
}
async function disable() {
  setBusy(true);
  try {
    await timeoutPromise(invoke("disable_write_blocker", { device }), 10000);
    await checkBlockerStatus();
    setMsg("OK: Write blocker disabled");
  } catch(e) { setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
</script>

<div>
  <h3>Write Blocker</h3>
  <div class="row">
    <label>Device: <input type="text" bind:value={device} placeholder="/dev/sda" disabled={busy} /></label>
  </div>
  <div class="status" class:active={enabled}>{enabled ? "🟢 Enabled" : "🔴 Disabled"}</div>
  <div class="method-info">
    <span class="method-label">Platform:</span>
    <span class="method-value">{platform || "Unknown"}</span>
    <span class="method-sep">·</span>
    <span class="method-label">Method:</span>
    <span class="method-value">{blockerMethod}</span>
  </div>
  <div class="actions">
    <button onclick={enable} class="btn-primary" disabled={busy||!device}>Enable</button>
    <button onclick={disable} class="btn-danger" disabled={busy||!enabled}>Disable</button>
    <button onclick={checkBlockerStatus} class="btn-sm" disabled={busy||!device}>Refresh</button>
  </div>
  {#if blockerNotes}
    <p class="note">{blockerNotes}</p>
  {/if}
  <p class="note">Platform: {platform}</p>

  <GuideCard title={writeBlockerGuide.title} icon={writeBlockerGuide.icon} steps={writeBlockerGuide.steps} references={writeBlockerGuide.references} />
</div>

<style>
h3 { margin: 0 0 16px; font-size: 16px; }
.row { margin-bottom: 12px; }
input { background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; width: 300px; }
.status { padding: 10px; border-radius: 8px; font-size: 14px; font-weight: 600; margin: 10px 0; background: rgba(239,68,68,0.1); border: 1px solid var(--danger); color: var(--danger); }
.status.active { background: rgba(34,197,94,0.1); border: 1px solid var(--success); color: var(--success); }
.method-info { display: flex; align-items: center; gap: 4px; margin-bottom: 12px; font-size: 12px; }
.method-label { color: var(--text-muted); }
.method-value { color: var(--text-secondary); font-family: var(--mono); }
.method-sep { color: var(--border); }
.actions { display: flex; gap: 10px; margin: 16px 0; }
.btn-primary, .btn-danger { padding: 10px 20px; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; }
.btn-primary { background: var(--primary); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger { background: var(--danger); }
.btn-sm { padding: 8px 14px; background: var(--btn-secondary-bg); color: var(--btn-secondary-text); border: 1px solid var(--border); border-radius: 8px; cursor: pointer; font-size: 12px; }
.note { font-size: 11px; color: var(--text-secondary); margin-top: 12px; line-height: 1.4; }
</style>
