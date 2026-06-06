<script>
import { invoke, openPath, isPreviewError } from "../api/tauri.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { listCases } from "../api/case.js";
import { listSnapshots } from "../api/snapshot.js";
import { exportJson, exportMarkdown, exportZip, listExports } from "../api/export.js";
import { openInAnalysisloom } from "../api/bridge.js";
import { ok, err, warn } from "../messages.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let cases = $state([]);
let selectedCaseId = $state("");
let snapshots = $state([]);
let selectedSnapshotId = $state("");
let format = $state("json");
let includeDiff = $state(true);
let exports = $state([]);
let lastResult = $state(null);

const profiles = [
  { id: "json", label: "JSON Pack", desc: "Normalized evidence_pack.json" },
  { id: "markdown", label: "Markdown Report", desc: "Human-readable case_report.md" },
  { id: "zip", label: "ZIP Bundle", desc: "Full case folder archive" },
];

async function loadCases() {
  try {
    cases = await listCases();
    if (cases.length && !selectedCaseId) selectedCaseId = cases[0].case_id;
  } catch (e) {
    if (!isPreviewError(e)) setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
}

async function loadSnapshots() {
  if (!selectedCaseId) return;
  try {
    snapshots = await listSnapshots(selectedCaseId);
    if (snapshots.length && !selectedSnapshotId) selectedSnapshotId = snapshots[0].snapshot_id;
  } catch {
    snapshots = [];
  }
}

async function loadExports() {
  if (!selectedCaseId) return;
  try {
    exports = await listExports(selectedCaseId);
  } catch {
    exports = [];
  }
}

$effect(() => {
  loadCases();
});

$effect(() => {
  if (selectedCaseId) {
    loadSnapshots();
    loadExports();
  }
});

async function generateExport() {
  if (!selectedCaseId) {
    setMsg("WARN: Select a case first");
    return;
  }
  setBusy(true);
  try {
    if (format === "json") {
      if (!selectedSnapshotId) throw new Error("Select a snapshot for JSON export");
      lastResult = await timeoutPromise(exportJson(selectedCaseId, selectedSnapshotId), 60000);
    } else if (format === "markdown") {
      if (!selectedSnapshotId) throw new Error("Select a snapshot for Markdown export");
      lastResult = await timeoutPromise(
        exportMarkdown(selectedCaseId, selectedSnapshotId, includeDiff),
        60000
      );
    } else {
      lastResult = await timeoutPromise(exportZip(selectedCaseId), 120000);
    }
    setMsg(`OK: Export saved: ${lastResult.output_path}`);
    await loadExports();
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function sendAnalysis() {
  if (!selectedCaseId) return;
  setBusy(true);
  try {
    const msg = await timeoutPromise(openInAnalysisloom(selectedCaseId), 15000);
    setMsg(ok(msg));
  } catch (e) {
    setMsg(err(String(e)));
  }
  setBusy(false);
}

async function openFolder() {
  if (lastResult?.output_path) {
    const dir = lastResult.output_path.substring(0, lastResult.output_path.lastIndexOf("/"));
    try {
      await openPath(dir);
    } catch {
      setMsg(`📁 ${dir}`);
    }
  }
}
</script>

<div class="export-tab">
  <SectionHeader title="Export Bundle" subtitle="Generate handover packages for investigation teams" />

  <MacCard title="Case">
    <select bind:value={selectedCaseId} class="full">
      <option value="">— Select case —</option>
      {#each cases as c}
        <option value={c.case_id}>{c.title} ({c.case_id.slice(0, 8)}…)</option>
      {/each}
    </select>
    {#if !cases.length}
      <p class="hint">No cases yet — create one from System Snapshot or Chain of Custody.</p>
    {/if}
  </MacCard>

  {#if format !== "zip"}
    <MacCard title="Snapshot">
      <select bind:value={selectedSnapshotId} class="full" disabled={!snapshots.length}>
        <option value="">— Select snapshot —</option>
        {#each snapshots as s}
          <option value={s.snapshot_id}>{s.profile} — {s.started_at} ({s.status})</option>
        {/each}
      </select>
    </MacCard>
  {/if}

  <MacCard title="Format">
    <div class="format-grid">
      {#each profiles as p}
        <label class="format-card" class:selected={format === p.id}>
          <input type="radio" bind:group={format} value={p.id} />
          <span class="fmt-label">{p.label}</span>
          <span class="fmt-desc">{p.desc}</span>
        </label>
      {/each}
    </div>
    {#if format === "markdown"}
      <label class="check"><input type="checkbox" bind:checked={includeDiff} /> Include diff summary</label>
    {/if}
  </MacCard>

  <div class="action-row">
    <button class="btn-primary" onclick={generateExport} disabled={busy || !selectedCaseId}>
      Generate Export
    </button>
    {#if selectedCaseId}
      <button class="btn-secondary" onclick={sendAnalysis} disabled={busy}>Send to AnalysisLoom</button>
    {/if}
  </div>

  {#if lastResult}
    <MacCard title="Last Export">
      <div class="result-row"><span>Type</span><PillBadge variant="info" label={lastResult.export_type} /></div>
      <div class="result-row"><span>Path</span><code>{lastResult.output_path}</code></div>
      <div class="result-row"><span>Size</span><span>{(lastResult.size_bytes / 1024).toFixed(1)} KB</span></div>
      <div class="result-row"><span>SHA-256</span><code class="hash">{lastResult.sha256.slice(0, 16)}…</code></div>
      <button class="btn-sm" onclick={openFolder}>Open Folder</button>
    </MacCard>
  {/if}

  {#if exports.length}
    <MacCard title="Previous Exports">
      {#each exports as ex}
        <div class="export-item">
          <PillBadge variant="info" label={ex.export_type} />
          <code>{ex.output_path.split("/").pop()}</code>
          <span class="size">{(ex.size_bytes / 1024).toFixed(1)} KB</span>
        </div>
      {/each}
    </MacCard>
  {/if}
</div>

<style>
  .export-tab { max-width: 640px; }
  .full { width: 100%; background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; }
  .hint { margin: 0; font-size: 12px; color: var(--text-muted); }
  .format-grid { display: flex; flex-direction: column; gap: 8px; }
  .format-card {
    display: grid; grid-template-columns: auto 1fr; gap: 2px 10px;
    padding: 10px 12px; border: 1px solid var(--border); border-radius: 10px; cursor: pointer;
  }
  .format-card.selected { border-color: var(--primary); background: rgba(59,130,246,0.08); }
  .fmt-label { font-weight: 600; font-size: 13px; grid-column: 2; }
  .fmt-desc { font-size: 11px; color: var(--text-secondary); grid-column: 2; }
  .check { display: flex; gap: 8px; font-size: 13px; margin-top: 8px; }
  .btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; }
  .btn-primary:disabled { opacity: 0.5; }
  .action-row { display: flex; gap: 10px; flex-wrap: wrap; margin: 12px 0; align-items: center; }
  .btn-secondary { padding: 10px 18px; background: var(--btn-secondary-bg); color: var(--btn-secondary-text); border: 1px solid var(--border); border-radius: 10px; font-weight: 600; cursor: pointer; }
  .btn-secondary:disabled { opacity: 0.5; }
  .btn-sm { padding: 6px 12px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 6px; color: var(--btn-secondary-text); cursor: pointer; margin-top: 8px; }
  .result-row { display: flex; gap: 12px; align-items: center; font-size: 12px; margin-bottom: 6px; }
  .result-row span:first-child { color: var(--text-muted); min-width: 60px; }
  code { font-family: var(--mono); font-size: 11px; word-break: break-all; }
  .hash { color: var(--text-secondary); }
  .export-item { display: flex; gap: 10px; align-items: center; font-size: 12px; padding: 4px 0; }
  .size { color: var(--text-muted); margin-left: auto; }
</style>
