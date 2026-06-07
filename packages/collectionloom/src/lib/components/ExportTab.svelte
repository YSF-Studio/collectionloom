<script>
import { openPath, isPreviewError } from "../api/tauri.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { listCases } from "../api/case.js";
import { listSnapshots } from "../api/snapshot.js";
import { exportJson, exportMarkdown, exportZip, listExports } from "../api/export.js";
import { getLocale, subscribeLocale } from "../stores/locale.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let cases = $state([]);
let selectedCaseId = $state("");
let snapshots = $state([]);
let selectedSnapshotId = $state("");
let format = $state("json");
let includeDiff = $state(true);
let exports = $state([]);
let lastResult = $state(null);
let locale = $state(getLocale());

$effect(() => subscribeLocale((_, resolved) => {
  locale = resolved;
}));

const text = {
  en: {
    title: "Export Bundle",
    subtitle: "Generate handover packages for investigation teams",
    noCases: "No cases to export",
    createFirst: "Create a case from System Snapshot or Chain of Custody first.",
    goToSnapshot: "Go to System Snapshot",
    selectCase: "— Select case —",
    selectSnapshot: "— Select snapshot —",
    includeDiff: "Include diff summary",
    generate: "Generate Export",
    lastExport: "Last Export",
    previousExports: "Previous Exports",
    type: "Type",
    path: "Path",
    size: "Size",
    openFolder: "Open Folder",
    case: "Case",
    snapshot: "Snapshot",
    format: "Format",
    jsonPack: "JSON Pack",
    markdownReport: "Markdown Report",
    zipBundle: "ZIP Bundle",
  },
  id: {
    title: "Paket Ekspor",
    subtitle: "Buat paket serah terima untuk tim investigasi",
    noCases: "Tidak ada kasus untuk diekspor",
    createFirst: "Buat kasus dari System Snapshot atau Chain of Custody terlebih dahulu.",
    goToSnapshot: "Ke System Snapshot",
    selectCase: "— Pilih kasus —",
    selectSnapshot: "— Pilih snapshot —",
    includeDiff: "Sertakan ringkasan diff",
    generate: "Buat Ekspor",
    lastExport: "Ekspor Terakhir",
    previousExports: "Ekspor Sebelumnya",
    type: "Jenis",
    path: "Path",
    size: "Ukuran",
    openFolder: "Buka Folder",
    case: "Kasus",
    snapshot: "Snapshot",
    format: "Format",
    jsonPack: "Paket JSON",
    markdownReport: "Laporan Markdown",
    zipBundle: "Paket ZIP",
  },
};
function tr(key) { return text[locale]?.[key] || text.en[key] || key; }

const profiles = $derived([
  { id: "json", label: locale === "id" ? "Paket JSON" : "JSON Pack", desc: locale === "id" ? "evidence_pack.json yang dinormalisasi" : "Normalized evidence_pack.json" },
  { id: "markdown", label: locale === "id" ? "Laporan Markdown" : "Markdown Report", desc: locale === "id" ? "case_report.md yang mudah dibaca manusia" : "Human-readable case_report.md" },
  { id: "zip", label: locale === "id" ? "Paket ZIP" : "ZIP Bundle", desc: locale === "id" ? "Arsip penuh folder kasus" : "Full case folder archive" },
]);

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
    setMsg(locale === "id" ? "PERINGATAN: Pilih kasus terlebih dahulu" : "WARN: Select a case first");
    return;
  }
  setBusy(true);
  try {
    if (format === "json") {
      if (!selectedSnapshotId) throw new Error(locale === "id" ? "Pilih snapshot untuk ekspor JSON" : "Select a snapshot for JSON export");
      lastResult = await timeoutPromise(exportJson(selectedCaseId, selectedSnapshotId), 60000);
    } else if (format === "markdown") {
      if (!selectedSnapshotId) throw new Error(locale === "id" ? "Pilih snapshot untuk ekspor Markdown" : "Select a snapshot for Markdown export");
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

<div class="tab-content export-tab">
  <SectionHeader title={tr("title")} subtitle={tr("subtitle")} />

  {#if !cases.length}
    <div class="empty-state">
      <span class="icon">📦</span>
      <p>{tr("noCases")}</p>
      <p class="empty-hint">{tr("createFirst")}</p>
      <button class="btn-sm primary" onclick={() => window.__goTo?.("snapshot")}>{tr("goToSnapshot")}</button>
    </div>
  {/if}

    <MacCard title={tr("case")}>
    <select bind:value={selectedCaseId} class="full" disabled={!cases.length}>
      <option value="">{tr("selectCase")}</option>
      {#each cases as c}
        <option value={c.case_id}>{c.title} ({c.case_id.slice(0, 8)}…)</option>
      {/each}
    </select>
  </MacCard>

  {#if format !== "zip"}
    <MacCard title={tr("snapshot")}>
      <select bind:value={selectedSnapshotId} class="full" disabled={!snapshots.length}>
        <option value="">{tr("selectSnapshot")}</option>
        {#each snapshots as s}
          <option value={s.snapshot_id}>{s.profile} — {s.started_at} ({s.status})</option>
        {/each}
      </select>
    </MacCard>
  {/if}

    <MacCard title={tr("format")}>
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
      <label class="check"><input type="checkbox" bind:checked={includeDiff} /> {tr("includeDiff")}</label>
    {/if}
  </MacCard>

  <div class="action-row">
      <button class="btn-primary" onclick={generateExport} disabled={busy || !selectedCaseId}>
      {tr("generate")}
    </button>
  </div>

  {#if lastResult}
    <MacCard title={tr("lastExport")}>
      <div class="result-row"><span>{tr("type")}</span><PillBadge variant="info" label={lastResult.export_type} /></div>
      <div class="result-row"><span>{tr("path")}</span><code>{lastResult.output_path}</code></div>
      <div class="result-row"><span>{tr("size")}</span><span>{(lastResult.size_bytes / 1024).toFixed(1)} KB</span></div>
          <div class="result-row"><span>SHA-256</span><code class="hash">{lastResult.sha256.slice(0, 16)}…</code></div>
      <button class="btn-sm" onclick={openFolder}>{tr("openFolder")}</button>
    </MacCard>
  {/if}

  {#if exports.length}
    <MacCard title={tr("previousExports")}>
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
  .full { width: 100%; background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; }
  .empty-hint { font-size: 11px !important; color: var(--text-muted); }
  .btn-sm.primary { background: var(--primary); color: white; border-color: var(--primary); padding: 6px 12px; border-radius: 6px; cursor: pointer; font-size: 12px; border: 1px solid var(--primary); }
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
  .btn-sm { padding: 6px 12px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 6px; color: var(--btn-secondary-text); cursor: pointer; margin-top: 8px; }
  .result-row { display: flex; gap: 12px; align-items: center; font-size: 12px; margin-bottom: 6px; }
  .result-row span:first-child { color: var(--text-muted); min-width: 60px; }
  code { font-family: var(--mono); font-size: 11px; word-break: break-all; }
  .hash { color: var(--text-secondary); }
  .export-item { display: flex; gap: 10px; align-items: center; font-size: 12px; padding: 4px 0; }
  .size { color: var(--text-muted); margin-left: auto; }
</style>
