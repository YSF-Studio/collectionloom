<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { snapshotGuide } from "../guides.js";
import { createCase, listCases } from "../api/case.js";
import { startSnapshot, getSnapshotProgress, listSnapshots } from "../api/snapshot.js";
import { compareSnapshots } from "../api/compare.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

const profiles = [
  { id: "triage_5m", label: "Triage 5m", desc: "System, process, network, autoruns, users — ~5 min" },
  { id: "ir_30m", label: "IR 30m", desc: "Extended capture with log excerpts" },
  { id: "deep_capture", label: "Deep Capture", desc: "Full collection with extended timeouts" },
];

let cases = $state([]);
let selectedCaseId = $state("");
let newCaseTitle = $state("");
let showNewCase = $state(false);
let selectedProfile = $state("triage_5m");
let running = $state(false);
let progress = $state(null);
let lastSnapshot = $state(null);
let snapshots = $state([]);
let compareA = $state("");
let compareB = $state("");
let compareResult = $state(null);
let pollId = $state(null);

async function loadCases() {
  try {
    cases = await listCases();
    if (cases.length && !selectedCaseId) selectedCaseId = cases[0].case_id;
  } catch {
    cases = [];
  }
}

async function loadSnapshots() {
  if (!selectedCaseId) return;
  try {
    snapshots = await listSnapshots(selectedCaseId);
  } catch {
    snapshots = [];
  }
}

$effect(() => {
  loadCases();
});

$effect(() => {
  if (selectedCaseId) loadSnapshots();
});

async function createNewCase() {
  if (!newCaseTitle.trim()) {
    setMsg("⚠️ Case title required");
    return;
  }
  setBusy(true);
  try {
    const c = await createCase({
      title: newCaseTitle,
      operator: "Investigator",
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      purpose: "System snapshot collection",
    });
    cases = [...cases, c];
    selectedCaseId = c.case_id;
    newCaseTitle = "";
    showNewCase = false;
    setMsg(`✅ Case created: ${c.title}`);
  } catch (e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function runSnapshot() {
  if (!selectedCaseId) {
    setMsg("⚠️ Select or create a case first");
    return;
  }
  running = true;
  setBusy(true);
  try {
    const meta = await timeoutPromise(startSnapshot(selectedCaseId, selectedProfile), 300000);
    lastSnapshot = meta;
    setMsg(`✅ Snapshot ${meta.status}: ${meta.snapshot_id.slice(0, 8)}…`);
    await loadSnapshots();
  } catch (e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  running = false;
  setBusy(false);
  if (pollId) clearInterval(pollId);
}

async function runCompare() {
  if (!compareA || !compareB) {
    setMsg("⚠️ Select two snapshots");
    return;
  }
  setBusy(true);
  try {
    compareResult = await timeoutPromise(
      compareSnapshots(selectedCaseId, compareA, compareB),
      60000
    );
    const s = compareResult.summary;
    setMsg(`✅ Compare: +${s?.total_added || 0} / -${s?.total_removed || 0} / ~${s?.total_changed || 0}`);
  } catch (e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

function statusVariant(status) {
  if (status === "completed" || status === "success") return "active";
  if (status === "partial") return "warning";
  if (status === "failed" || status === "error") return "inactive";
  return "info";
}
</script>

<div class="snapshot-tab">
  <SectionHeader title="System Snapshot" subtitle="Modular collector — capture and compare system state" />

  <MacCard title="Case">
    <div class="row">
      <select bind:value={selectedCaseId} class="full">
        <option value="">— Select case —</option>
        {#each cases as c}
          <option value={c.case_id}>{c.title}</option>
        {/each}
      </select>
      <button class="btn-sm" onclick={() => (showNewCase = !showNewCase)}>New Case</button>
    </div>
    {#if showNewCase}
      <div class="row">
        <input bind:value={newCaseTitle} placeholder="Case title" class="full" />
        <button class="btn-sm" onclick={createNewCase}>Create</button>
      </div>
    {/if}
  </MacCard>

  <MacCard title="Capture Profile">
    <div class="profile-grid">
      {#each profiles as p}
        <label class="profile-card" class:selected={selectedProfile === p.id}>
          <input type="radio" bind:group={selectedProfile} value={p.id} disabled={running} />
          <span class="p-label">{p.label}</span>
          <span class="p-desc">{p.desc}</span>
        </label>
      {/each}
    </div>
    <button class="btn-primary" onclick={runSnapshot} disabled={running || busy || !selectedCaseId}>
      {running ? "Capturing…" : "Start Snapshot"}
    </button>
  </MacCard>

  {#if lastSnapshot?.modules}
    <MacCard title="Collector Results">
      {#each lastSnapshot.modules as mod}
        <div class="mod-row">
          <span class="mod-name">{mod.name}</span>
          <PillBadge variant={statusVariant(mod.status)} label={mod.status} />
          <span class="mod-meta">{mod.items_count ?? 0} items · {mod.duration_ms ?? 0}ms</span>
          {#if mod.error}<span class="mod-err">{mod.error}</span>{/if}
        </div>
      {/each}
      <p class="integrity">Integrity: <code>{lastSnapshot.integrity_hash?.slice(0, 20)}…</code></p>
    </MacCard>
  {/if}

  {#if snapshots.length >= 2}
    <MacCard title="Compare Snapshots">
      <div class="row">
        <select bind:value={compareA} class="full">
          <option value="">Snapshot A (baseline)</option>
          {#each snapshots as s}
            <option value={s.snapshot_id}>{s.started_at} — {s.profile}</option>
          {/each}
        </select>
        <select bind:value={compareB} class="full">
          <option value="">Snapshot B (current)</option>
          {#each snapshots as s}
            <option value={s.snapshot_id}>{s.started_at} — {s.profile}</option>
          {/each}
        </select>
      </div>
      <button class="btn-sm" onclick={runCompare} disabled={busy}>Run Compare</button>
      {#if compareResult?.summary}
        <div class="compare-summary">
          Added: {compareResult.summary.total_added} ·
          Removed: {compareResult.summary.total_removed} ·
          Changed: {compareResult.summary.total_changed}
        </div>
      {/if}
    </MacCard>
  {/if}

  <GuideCard title={snapshotGuide.title} icon={snapshotGuide.icon} steps={snapshotGuide.steps} references={snapshotGuide.references} />
</div>

<style>
  .snapshot-tab { max-width: 720px; }
  .row { display: flex; gap: 8px; align-items: center; margin-bottom: 8px; }
  .full { flex: 1; background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; font-size: 13px; }
  .profile-grid { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
  .profile-card {
    display: grid; grid-template-columns: auto 1fr; gap: 2px 10px;
    padding: 10px 12px; border: 1px solid var(--border); border-radius: 10px; cursor: pointer;
  }
  .profile-card.selected { border-color: var(--primary); background: rgba(59,130,246,0.08); }
  .p-label { font-weight: 600; font-size: 13px; grid-column: 2; }
  .p-desc { font-size: 11px; color: var(--text-secondary); grid-column: 2; }
  .btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; }
  .btn-primary:disabled { opacity: 0.5; }
  .btn-sm { padding: 6px 14px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 8px; color: var(--btn-secondary-text); cursor: pointer; font-size: 12px; }
  .mod-row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; padding: 6px 0; border-bottom: 1px solid rgba(255,255,255,0.04); font-size: 12px; }
  .mod-name { font-weight: 600; min-width: 80px; text-transform: capitalize; }
  .mod-meta { color: var(--text-muted); }
  .mod-err { color: var(--danger); font-size: 11px; width: 100%; }
  .integrity { margin: 8px 0 0; font-size: 11px; color: var(--text-secondary); }
  .compare-summary { margin-top: 10px; font-size: 12px; color: var(--primary); }
  code { font-family: var(--mono); }
</style>
