<script>
import { invoke } from "../api/tauri.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let report = $state(null);
let loading = $state(false);

const categoryLabels = {
  pure_rust: "Pure Rust (built-in)",
  system_library: "System library",
  external_binary: "External binary",
  privilege: "OS privilege",
};

async function runCheck() {
  loading = true;
  setBusy(true);
  try {
    report = await timeoutPromise(invoke("run_preflight_check"), 15000);
    if (report?.missingCount > 0) {
      setMsg(`WARN: ${report.missingCount} tool(s) missing — see install hints below`);
    } else if (report?.warningCount > 0) {
      setMsg(`WARN: ${report.summary}`);
    } else {
      setMsg("OK: All prerequisites detected");
    }
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  loading = false;
  setBusy(false);
}

function groupedChecks(checks) {
  const order = ["pure_rust", "system_library", "external_binary", "privilege"];
  const groups = {};
  for (const c of checks || []) {
    const key = c.category;
    if (!groups[key]) groups[key] = [];
    groups[key].push(c);
  }
  return order.filter((k) => groups[k]?.length).map((k) => ({ key: k, items: groups[k] }));
}

$effect(() => {
  runCheck();
});
</script>

<div class="tab-content preflight-tab">
  <SectionHeader
    title="Prerequisites"
    subtitle="Detect installed tools, system libraries, and privilege level before acquisition"
  />

  <div class="toolbar">
    <button class="btn-sm" onclick={runCheck} disabled={busy || loading}>
      {#if loading}<span class="spinner">↻</span>{/if}
      {loading ? "Checking…" : "Re-run Pre-flight Check"}
    </button>
  </div>

  {#if loading && !report}
    <MacCard title="Running checks">
      <p class="hint">Scanning ./tools/, PATH, system libraries, and privileges…</p>
    </MacCard>
  {:else if report}
    <MacCard title="Portable Kit">
      <div class="summary-row">
        <PillBadge
          variant={report.portable?.portableMode ? "active" : "info"}
          label={report.portable?.portableMode ? "Portable mode" : "Standard install"}
        />
        {#if report.portable?.kitRoot}
          <span class="platform kit-path" title={report.portable.kitRoot}>Kit: {report.portable.kitRoot}</span>
        {/if}
      </div>
      <p class="summary-text">
        {#if report.portable?.toolsDirExists}
          Tools folder: <code>{report.portable.toolsDir}</code>
          {#if report.portable.manifestLoaded} · manifest.json loaded{/if}
        {:else}
          No <code>./tools/</code> folder — copy avml, adb, etc. beside the app on forensic USB for zero-install field use.
        {/if}
      </p>
    </MacCard>

    <MacCard title="Summary">
      <div class="summary-row">
        <PillBadge
          variant={report.missingCount === 0 && report.warningCount === 0 ? "active" : "warning"}
          label={report.missingCount === 0 && report.warningCount === 0 ? "Ready" : "Action needed"}
        />
        <span class="platform">{report.platform} · {report.checkedAt?.slice(0, 19).replace("T", " ")}</span>
      </div>
      <p class="summary-text">{report.summary}</p>
      {#if report.missingCount > 0}
        <p class="warn-note">
          External tools (RAM, mobile, network) must be installed separately — CollectionLoom cannot ship them due to licensing and maintenance. See install hints per row.
        </p>
      {/if}
    </MacCard>

    {#each groupedChecks(report.checks) as group}
      <MacCard title={categoryLabels[group.key] || group.key}>
        <ul class="check-list">
          {#each group.items as item}
            <li class="check-row" class:missing={!item.available && group.key !== "pure_rust"}>
              <div class="check-head">
                <PillBadge
                  variant={item.available ? "active" : group.key === "privilege" ? "warning" : "inactive"}
                  label={item.available ? "OK" : "Missing"}
                />
                <strong>{item.name}</strong>
                <span class="for">→ {item.requiredFor}</span>
              </div>
              <p class="detail">{item.detail}</p>
              {#if item.installHint && !item.available}
                <p class="hint install">{item.installHint}</p>
              {/if}
            </li>
          {/each}
        </ul>
      </MacCard>
    {/each}
  {/if}
</div>

<style>
  .toolbar { margin-bottom: 12px; }
  .summary-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
    flex-wrap: wrap;
  }
  .platform { font-size: 12px; color: var(--text-muted); }
  .kit-path {
    max-width: 360px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  code { font-size: 11px; }
  .summary-text { font-size: 13px; color: var(--text-secondary); margin: 0; }
  .warn-note {
    font-size: 12px;
    color: var(--warn);
    margin: 10px 0 0;
    line-height: 1.45;
  }
  .check-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .check-row {
    padding: 10px 12px;
    border-radius: 8px;
    background: var(--card-bg, rgba(0, 0, 0, 0.03));
    border: 1px solid var(--border);
  }
  .check-row.missing {
    border-color: var(--warn);
  }
  .check-head {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 4px;
  }
  .check-head strong { font-size: 13px; }
  .for { font-size: 11px; color: var(--text-muted); }
  .detail { font-size: 12px; color: var(--text-secondary); margin: 0; }
  .install { margin: 6px 0 0; font-size: 11px; }
  .spinner { display: inline-block; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
