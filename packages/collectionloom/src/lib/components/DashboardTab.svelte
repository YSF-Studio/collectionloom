<script>
import { openPath } from "../api/tauri.js";
import { listCaseSummaries } from "../api/case.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { err, warn } from "../messages.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let summaries = $state([]);

async function load() {
  try {
    summaries = await listCaseSummaries();
  } catch (e) {
    if (!isPreviewError(e)) setMsg(err(String(e)));
  }
}

$effect(() => {
  load();
});

async function openCaseFolder(dir) {
  try {
    await openPath(dir);
  } catch {
    setMsg(warn(`Case folder: ${dir}`));
  }
}

function statusVariant(status) {
  if (status === "open") return "active";
  if (status === "closed") return "inactive";
  return "info";
}
</script>

<div class="dashboard">
  <SectionHeader
    title="Case Dashboard"
    subtitle="Overview of all investigations in ~/CollectionLoom/cases/"
  />

  <div class="toolbar">
    <button class="btn-sm" onclick={load} disabled={busy}>Refresh</button>
  </div>

  {#if !summaries.length}
    <MacCard title="No cases yet">
      <p class="hint">Create a case from System Snapshot or Chain of Custody to get started.</p>
    </MacCard>
  {:else}
    {#each summaries as row}
      <MacCard title={row.case.title}>
        <div class="meta">
          <PillBadge variant={statusVariant(row.case.status)} label={row.case.status} />
          <span class="id">{row.case.case_id.slice(0, 8)}…</span>
          <span class="date">{row.case.created_at?.slice(0, 10)}</span>
        </div>
        <div class="stats">
          <div><strong>{row.snapshot_count}</strong><span>Snapshots</span></div>
          <div><strong>{row.export_count}</strong><span>Exports</span></div>
          <div><strong>{row.diff_count}</strong><span>Diffs</span></div>
        </div>
        <p class="operator">Operator: {row.case.operator?.name || "—"}</p>
        <div class="actions">
          <button class="btn-sm" onclick={() => openCaseFolder(row.case_dir)}>Open Folder</button>
        </div>
      </MacCard>
    {/each}
  {/if}
</div>

<style>
  .dashboard { max-width: 720px; }
  .toolbar { margin-bottom: 12px; }
  .hint { margin: 0; font-size: 13px; color: var(--text-muted); }
  .meta { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; flex-wrap: wrap; }
  .id { font-family: var(--mono); font-size: 11px; color: var(--text-secondary); }
  .date { font-size: 11px; color: var(--text-muted); margin-left: auto; }
  .stats {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin: 10px 0;
  }
  .stats div {
    text-align: center; padding: 8px; border-radius: 8px;
    background: var(--input-bg); border: 1px solid var(--border);
  }
  .stats strong { display: block; font-size: 18px; color: var(--primary); }
  .stats span { font-size: 10px; color: var(--text-muted); text-transform: uppercase; }
  .operator { font-size: 12px; color: var(--text-secondary); margin: 0 0 10px; }
  .actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .btn-sm {
    padding: 6px 12px; background: var(--btn-secondary-bg); color: var(--btn-secondary-text);
    border: 1px solid var(--border); border-radius: 6px; cursor: pointer; font-size: 12px;
  }
  .btn-sm.primary { background: var(--primary); color: white; border-color: var(--primary); }
  .btn-sm:disabled { opacity: 0.5; }
</style>
