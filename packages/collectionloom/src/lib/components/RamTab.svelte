<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import { ramCaptureGuide } from "../guides.js";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();
let tools = $state([]);
let selectedTool = $state("");
let outputPath = $state("/mnt/evidence/ram_capture.lime");
let compress = $state(true);
let progress = $state(null);
let ramSize = $state(null);

// Auto hash
let autoHash = $state(true);
let hashResult = $state("");

// Process list
let processList = $state([]);
let showProcesses = $state(false);

async function listTools() {
  setBusy(true);
  try { tools = await timeoutPromise(invoke("list_ram_tools"), 5000); } catch(e) {}
  try { ramSize = await timeoutPromise(invoke("get_ram_size"), 5000); } catch(e) {}
  setBusy(false);
}
async function capture() {
  setBusy(true);
  hashResult = "";
  try {
    const result = await timeoutPromise(invoke("capture_ram", { tool: selectedTool, output: outputPath, compress }), 120000);
    setMsg(`OK: ${result}`);
    // Auto hash after successful capture
    if (autoHash && outputPath) {
      try {
        const hash = await timeoutPromise(invoke("compute_file_hash", { path: outputPath }), 30000);
        hashResult = `SHA-256: ${hash}`;
      } catch(e) {
        hashResult = `❌ Hash computation failed: ${typeof e === "string" ? e : String(e)}`;
      }
    }
  } catch(e) { setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}

async function listProcesses() {
  showProcesses = true;
  try {
    processList = await timeoutPromise(invoke("list_processes"), 10000);
  } catch(e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
}

async function refreshProcesses() {
  try {
    processList = await timeoutPromise(invoke("list_processes"), 10000);
  } catch(e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
}

$effect(() => { listTools(); });
</script>

<div>
  <h3>RAM Capture</h3>
  {#if ramSize}<p class="info">System RAM: {(ramSize/1e9).toFixed(1)} GB</p>{/if}
  <div class="row">
    <label>Tool: <select bind:value={selectedTool} disabled={busy}>
      <option value="">-- Select --</option>
      {#each tools as tool}<option value={tool}>{tool}</option>{/each}
    </select></label>
  </div>
  <div class="row">
    <label>Output: <input type="text" bind:value={outputPath} disabled={busy} /></label>
    <label><input type="checkbox" bind:checked={compress} disabled={busy} /> Compress</label>
  </div>
  <div class="row">
    <label><input type="checkbox" bind:checked={autoHash} disabled={busy} /> Auto hash after capture</label>
  </div>
  <div class="actions">
    <button onclick={capture} class="btn-primary" disabled={busy||!selectedTool}>▶ Capture RAM</button>
    <button onclick={listProcesses} class="btn-sm" disabled={busy}>List Processes</button>
  </div>

  {#if hashResult}
  <div class="hash-result">{hashResult}</div>
  {/if}

  {#if showProcesses && processList.length > 0}
  <div class="process-section">
    <div class="process-header">
      <span>Running Processes ({processList.length})</span>
      <button onclick={refreshProcesses} class="btn-sm">Refresh</button>
    </div>
    <div class="process-list">
      {#each processList as proc}
        <div class="process-item">{proc.pid} | {proc.name} | {(proc.memory_bytes / 1048576).toFixed(1)} MB | {proc.cpu_percent.toFixed(1)}% CPU</div>
      {/each}
    </div>
  </div>
  {/if}

  <GuideCard title={ramCaptureGuide.title} icon={ramCaptureGuide.icon} steps={ramCaptureGuide.steps} references={ramCaptureGuide.references} />
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.info { font-size:12px; color:var(--text-secondary); margin-bottom:10px; }
.row { display:flex; gap:10px; align-items:center; margin-bottom:12px; }
select, input { background: var(--input-bg); color: var(--text); border:1px solid var(--border); border-radius:6px; padding:6px 10px; }
.actions { display:flex; gap:8px; align-items:center; margin-bottom:12px; }
.btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary:disabled { opacity:0.5; }
.btn-sm { padding:5px 10px; background:var(--border); color:#e0e0e0; border:none; border-radius:6px; cursor:pointer; font-size:12px; }
.btn-sm:disabled { opacity:0.5; cursor:not-allowed; }
.hash-result { background: var(--input-bg); border:1px solid var(--border); border-radius:6px; padding:8px 12px; font-size:12px; color:var(--primary); margin-bottom:12px; font-family:monospace; }
.process-section { background: var(--input-bg); border:1px solid var(--border); border-radius:8px; padding:12px; margin-bottom:12px; }
.process-header { display:flex; justify-content:space-between; align-items:center; margin-bottom:8px; font-size:12px; color:var(--text-secondary); }
.process-list { max-height:200px; overflow-y:auto; }
.process-item { font-size:11px; padding:4px 0; border-bottom:1px solid var(--border); color:#ccc; font-family:monospace; }
</style>
