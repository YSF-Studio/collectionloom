<script>
import { invoke } from "@tauri-apps/api/core";
import GuideCard from "./GuideCard.svelte";
import { snapshotGuide } from "../guides.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();
let loading = $state(false);
let baselineId = $state(null);
let postExecId = $state(null);
let baselineSnapshot = $state(null);
let postExecSnapshot = $state(null);
let compareResult = $state(null);

async function takeBaseline() {
  loading = true; setBusy(true);
  try {
    const result = await timeoutPromise(invoke("take_snapshot"), 60000);
    baselineId = result.id;
    baselineSnapshot = result;
    setMsg("✅ Baseline snapshot captured");
  } catch(e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  loading = false; setBusy(false);
}

async function takePostExecution() {
  loading = true; setBusy(true);
  try {
    const result = await timeoutPromise(invoke("take_snapshot"), 60000);
    postExecId = result.id;
    postExecSnapshot = result;
    setMsg("✅ Post-execution snapshot captured");
  } catch(e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  loading = false; setBusy(false);
}

async function compareSnapshots() {
  if (!baselineId) return;
  loading = true; setBusy(true);
  try {
    const diff = await timeoutPromise(invoke("compare_snapshot", {
      previousId: baselineId
    }), 60000);
    compareResult = diff;
    setMsg(`✅ Diff complete — ${diff.changes || diff.new_files + diff.deleted_files + diff.modified_files || "0"} changes detected`);
  } catch(e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  loading = false; setBusy(false);
}

function riskColor(level) {
  if (!level) return "#555";
  if (level === "LOW") return "#22c55e";
  if (level === "MEDIUM" || level === "MED") return "#f59e0b";
  if (level === "HIGH") return "#ef4444";
  if (level === "CRITICAL") return "#dc2626";
  return "#ef4444";
}

function riskBgColor(level) {
  if (!level) return "transparent";
  if (level === "LOW") return "rgba(34,197,94,0.1)";
  if (level === "MEDIUM" || level === "MED") return "rgba(245,158,11,0.1)";
  if (level === "HIGH") return "rgba(239,68,68,0.1)";
  if (level === "CRITICAL") return "rgba(220,38,38,0.15)";
  return "transparent";
}
</script>

<div class="snapshot-tab">
  <h3>📸 System Snapshot</h3>
  <p class="desc">Capture point-in-time system state for forensic comparison and integrity verification.</p>

  <div class="card">
    <h4>Take Baseline Snapshot</h4>
    <p style="color:var(--text-secondary);font-size:12px;margin:0 0 12px">
      Captures filesystem state, running processes, network connections, and system information before executing tools.
    </p>
    <button class="btn-primary" onclick={takeBaseline} disabled={loading || busy}>
      {loading && !baselineId ? "⏳ Capturing..." : "📸 Take Baseline Snapshot #1"}
    </button>
    {#if baselineSnapshot}
      <div class="snap-info" style="margin-top:10px">
        <span class="snap-badge baseline">Baseline: {baselineSnapshot.id}</span>
        {#if baselineSnapshot.timestamp}
          <span class="snap-time">{baselineSnapshot.timestamp}</span>
        {/if}
        <span class="snap-count">{baselineSnapshot.file_count || 0} files, {baselineSnapshot.process_count || 0} processes</span>
      </div>
    {/if}
  </div>

  <div class="card">
    <h4>Take Post-Execution Snapshot</h4>
    <p style="color:var(--text-secondary);font-size:12px;margin:0 0 12px">
      Captures a second snapshot after tool execution to detect changes.
    </p>
    <button class="btn-primary" onclick={takePostExecution} disabled={loading || busy || !baselineId}>
      {loading && baselineId && !postExecId ? "⏳ Capturing..." : "📸 Take Post-Execution Snapshot #2"}
    </button>
    {#if !baselineId}
      <p style="font-size:11px;color:#555;margin:8px 0 0">Take baseline first</p>
    {/if}
    {#if postExecSnapshot}
      <div class="snap-info" style="margin-top:10px">
        <span class="snap-badge postexec">Post: {postExecSnapshot.id}</span>
        {#if postExecSnapshot.timestamp}
          <span class="snap-time">{postExecSnapshot.timestamp}</span>
        {/if}
        <span class="snap-count">{postExecSnapshot.file_count || 0} files, {postExecSnapshot.process_count || 0} processes</span>
      </div>
    {/if}
  </div>

  {#if baselineId}
    <div class="card compare-card">
      <h4>Compare Snapshots</h4>
      <p style="color:var(--text-secondary);font-size:12px;margin:0 0 12px">
        Compare baseline with post-execution snapshot to identify changes.
      </p>
      <button class="btn-compare" onclick={compareSnapshots} disabled={loading || busy || !postExecId}>
        {loading && compareResult === null ? "⏳ Comparing..." : "🔍 Compare Snapshots"}
      </button>
      {#if !postExecId}
        <p style="font-size:11px;color:#555;margin:8px 0 0">Take post-execution snapshot first</p>
      {/if}
    </div>
  {/if}

  {#if compareResult}
    <div class="card diff-card">
      <h4>Diff Results</h4>

      <!-- Risk Level Indicator -->
      {#if compareResult.risk_level}
        <div class="risk-indicator" style="background:{riskBgColor(compareResult.risk_level)};border:1px solid {riskColor(compareResult.risk_level)}">
          <span class="risk-label">Risk Level</span>
          <span class="risk-value" style="color:{riskColor(compareResult.risk_level)}">{compareResult.risk_level}</span>
        </div>
      {/if}

      <div class="diff-stats">
        <div class="stat-card stat-new">
          <span class="stat-num">{compareResult.new_files || 0}</span>
          <span class="stat-label">New Files</span>
        </div>
        <div class="stat-card stat-deleted">
          <span class="stat-num">{compareResult.deleted_files || 0}</span>
          <span class="stat-label">Deleted Files</span>
        </div>
        <div class="stat-card stat-modified">
          <span class="stat-num">{compareResult.modified_files || 0}</span>
          <span class="stat-label">Modified Files</span>
        </div>
        <div class="stat-card stat-procs">
          <span class="stat-num">{compareResult.new_processes || 0}</span>
          <span class="stat-label">New Processes</span>
        </div>
        <div class="stat-card stat-net">
          <span class="stat-num">{compareResult.new_connections || 0}</span>
          <span class="stat-label">New Connections</span>
        </div>
      </div>

      {#if compareResult.new_files && compareResult.new_files > 0}
        <div class="diff-section">
          <h5 class="diff-heading new-heading">📄 New Files</h5>
          <div class="diff-items">
            {#if Array.isArray(compareResult.new_file_list)}
              {#each compareResult.new_file_list as f}
                <div class="diff-item new-item">{f}</div>
              {/each}
            {:else}
              <span class="diff-count">{compareResult.new_files} new file(s) detected</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if compareResult.deleted_files && compareResult.deleted_files > 0}
        <div class="diff-section">
          <h5 class="diff-heading del-heading">🗑️ Deleted Files</h5>
          <div class="diff-items">
            {#if Array.isArray(compareResult.deleted_file_list)}
              {#each compareResult.deleted_file_list as f}
                <div class="diff-item del-item">{f}</div>
              {/each}
            {:else}
              <span class="diff-count">{compareResult.deleted_files} deleted file(s)</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if compareResult.modified_files && compareResult.modified_files > 0}
        <div class="diff-section">
          <h5 class="diff-heading mod-heading">✏️ Modified Files</h5>
          <div class="diff-items">
            {#if Array.isArray(compareResult.modified_file_list)}
              {#each compareResult.modified_file_list as f}
                <div class="diff-item mod-item">{f}</div>
              {/each}
            {:else}
              <span class="diff-count">{compareResult.modified_files} modified file(s)</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if compareResult.new_processes && compareResult.new_processes > 0}
        <div class="diff-section">
          <h5 class="diff-heading proc-heading">⚙️ New Processes</h5>
          <div class="diff-items">
            {#if Array.isArray(compareResult.new_process_list)}
              {#each compareResult.new_process_list as p}
                <div class="diff-item proc-item">{p}</div>
              {/each}
            {:else}
              <span class="diff-count">{compareResult.new_processes} new process(es)</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if compareResult.new_connections && compareResult.new_connections > 0}
        <div class="diff-section">
          <h5 class="diff-heading net-heading">🌐 New Network Connections</h5>
          <div class="diff-items">
            {#if Array.isArray(compareResult.new_connection_list)}
              {#each compareResult.new_connection_list as c}
                <div class="diff-item net-item">{c}</div>
              {/each}
            {:else}
              <span class="diff-count">{compareResult.new_connections} new connection(s)</span>
            {/if}
          </div>
        </div>
      {/if}

      {#if compareResult.report}
        <pre class="diff-report">{compareResult.report}</pre>
      {/if}
    </div>
  {/if}

  {#if !baselineId && !loading}
    <div class="empty-state">
      <span class="icon">📸</span>
      <p>No snapshots taken yet</p>
      <span style="font-size:11px;color:var(--text-muted)">Start by capturing a baseline snapshot</span>
    </div>
  {/if}

  <!-- GuideCard -->
  <GuideCard title={snapshotGuide.title} icon={snapshotGuide.icon} steps={snapshotGuide.steps} references={snapshotGuide.references} />
</div>

<style>
.snapshot-tab { max-width: 700px; }
.snapshot-tab h3 { margin: 0 0 4px; font-size: 16px; }
.desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 20px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px 20px; margin-bottom: 14px; }
.card h4 { margin: 0 0 8px; font-size: 13px; color: #ccc; }
.btn { padding: 6px 14px; border: 1px solid var(--border); background: var(--card); color: var(--text-secondary); border-radius: 6px; cursor: pointer; font-size: 12px; }
.btn:hover { background: var(--card-hover); color: var(--text); }
.btn-primary { padding: 8px 18px; background: var(--primary); color: white; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; cursor: pointer; }
.btn-primary:disabled { opacity: 0.4; cursor: default; }

.btn-compare {
  padding: 10px 22px;
  background: rgba(245,158,11,0.15);
  color: #f59e0b;
  border: 1px solid #f59e0b;
  border-radius: 8px;
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-compare:hover:not(:disabled) {
  background: rgba(245,158,11,0.25);
}
.btn-compare:disabled {
  opacity: 0.4;
  cursor: default;
}

.snap-info {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  font-size: 11px;
}
.snap-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
  font-family: var(--mono, monospace);
}
.snap-badge.baseline {
  background: rgba(59,130,246,0.15);
  color: #3b82f6;
  border: 1px solid rgba(59,130,246,0.3);
}
.snap-badge.postexec {
  background: rgba(245,158,11,0.15);
  color: #f59e0b;
  border: 1px solid rgba(245,158,11,0.3);
}
.snap-time {
  color: var(--text-secondary);
}
.snap-count {
  color: var(--text-secondary);
}

.compare-card {
  border-left: 3px solid #f59e0b;
}

.result-card { border-left: 3px solid var(--primary); }
.diff-card { border-left: 3px solid var(--warn); }

/* Risk Indicator */
.risk-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  margin-bottom: 14px;
}
.risk-label {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.risk-value {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: 1px;
}

/* Diff Stats Grid */
.diff-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 8px;
  margin-bottom: 16px;
}
.stat-card {
  text-align: center;
  padding: 12px 8px;
  border-radius: 8px;
  border: 1px solid var(--border);
}
.stat-card.stat-new { background: rgba(34,197,94,0.06); border-color: rgba(34,197,94,0.2); }
.stat-card.stat-deleted { background: rgba(239,68,68,0.06); border-color: rgba(239,68,68,0.2); }
.stat-card.stat-modified { background: rgba(245,158,11,0.06); border-color: rgba(245,158,11,0.2); }
.stat-card.stat-procs { background: rgba(59,130,246,0.06); border-color: rgba(59,130,246,0.2); }
.stat-card.stat-net { background: rgba(139,92,246,0.06); border-color: rgba(139,92,246,0.2); }

.stat-num {
  display: block;
  font-size: 24px;
  font-weight: 700;
  color: #e0e0e0;
  line-height: 1.2;
}
.stat-label {
  display: block;
  font-size: 10px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 2px;
}

/* Diff sections */
.diff-section {
  margin-bottom: 12px;
}
.diff-heading {
  font-size: 12px;
  font-weight: 600;
  margin: 0 0 4px;
}
.new-heading { color: #22c55e; }
.del-heading { color: #ef4444; }
.mod-heading { color: #f59e0b; }
.proc-heading { color: #3b82f6; }
.net-heading { color: #8b5cf6; }

.diff-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.diff-item {
  padding: 3px 8px;
  font-size: 11px;
  border-radius: 4px;
  font-family: var(--mono, monospace);
  word-break: break-all;
}
.new-item { background: rgba(34,197,94,0.06); color: #4ade80; }
.del-item { background: rgba(239,68,68,0.06); color: #f87171; }
.mod-item { background: rgba(245,158,11,0.06); color: #fbbf24; }
.proc-item { background: rgba(59,130,246,0.06); color: #60a5fa; }
.net-item { background: rgba(139,92,246,0.06); color: #a78bfa; }
.diff-count {
  font-size: 11px;
  color: var(--text-secondary);
  padding: 2px 8px;
}

.info-grid { display: grid; grid-template-columns: auto 1fr; gap: 4px 12px; font-size: 12px; }
.info-grid .key { color: var(--text-secondary); }
.info-grid .val { color: #d4d4d4; }
.mono { font-family: var(--mono); font-size: 11px; }
.diff-report { margin-top: 12px; padding: 10px; background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; font-size: 11px; color: #d4d4d4; overflow-x: auto; max-height: 300px; overflow-y: auto; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 160px; color: var(--text-muted); gap: 8px; }
.empty-state .icon { font-size: 36px; opacity: 0.3; }
.empty-state p { margin: 0; font-size: 14px; }
</style>
