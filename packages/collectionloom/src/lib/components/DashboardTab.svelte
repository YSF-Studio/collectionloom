<script>
import { openPath, isPreviewError } from "../api/tauri.js";
import { listCaseSummaries } from "../api/case.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { getResolvedLocale, subscribeLocale } from "../stores/locale.js";
import { err, warn } from "../messages.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let summaries = $state([]);
let loading = $state(false);
let locale = $state(getResolvedLocale());

const text = {
  en: {
    title: "Case Dashboard",
    subtitle: "Overview of all investigations in ~/CollectionLoom/cases/",
    loading: "Loading…",
    refresh: "Refresh",
    loadingCases: "Loading cases",
    fetching: "Fetching case summaries…",
    noCases: "No cases yet",
    emptyHint: "Create a case from System Snapshot or Chain of Custody to get started.",
    goToCoC: "Go to Chain of Custody",
    snapshots: "Snapshots",
    exports: "Exports",
    diffs: "Diffs",
    operator: "Operator",
    caseFolder: "Case folder",
  },
  id: {
    title: "Dasbor Kasus",
    subtitle: "Ringkasan semua investigasi di ~/CollectionLoom/cases/",
    loading: "Memuat…",
    refresh: "Muat Ulang",
    loadingCases: "Memuat kasus",
    fetching: "Mengambil ringkasan kasus…",
    noCases: "Belum ada kasus",
    emptyHint: "Buat kasus dari System Snapshot atau Chain of Custody untuk memulai.",
    goToCoC: "Ke Chain of Custody",
    snapshots: "Snapshot",
    exports: "Ekspor",
    diffs: "Diff",
    operator: "Operator",
    caseFolder: "Folder kasus",
  },
};

const t = (key) => text[locale]?.[key] ?? text.en[key] ?? key;

const unsubscribe = subscribeLocale((_, resolved) => {
  locale = resolved;
});

$effect(() => () => unsubscribe());

async function load() {
  loading = true;
  try {
    summaries = await listCaseSummaries();
  } catch (e) {
    if (!isPreviewError(e)) setMsg(err(String(e)));
  }
  loading = false;
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

<div class="tab-content dashboard">
  <SectionHeader
    title={t("title")}
    subtitle={t("subtitle")}
  />

  <div class="toolbar">
    <button class="btn-sm" onclick={load} disabled={busy || loading}>
      {#if loading}<span class="spinner">↻</span>{/if}
      {loading ? t("loading") : t("refresh")}
    </button>
  </div>

  {#if loading && !summaries.length}
    <MacCard title={t("loadingCases")}>
      <p class="hint">{t("fetching")}</p>
    </MacCard>
  {:else if !summaries.length}
    <div class="empty-state">
      <span class="icon">📁</span>
      <p>{t("noCases")}</p>
      <p class="empty-hint">{t("emptyHint")}</p>
      <button class="btn-sm primary" onclick={() => window.__goTo?.("coc")}>{t("goToCoC")}</button>
    </div>
  {:else}
    {#each summaries as row}
      <MacCard title={row.case.title}>
        <div class="meta">
          <PillBadge variant={statusVariant(row.case.status)} label={row.case.status} />
          <span class="id">{row.case.case_id.slice(0, 8)}…</span>
          <span class="date">{row.case.created_at?.slice(0, 10)}</span>
        </div>
        <div class="stats">
          <div><strong>{row.snapshot_count}</strong><span>{t("snapshots")}</span></div>
          <div><strong>{row.export_count}</strong><span>{t("exports")}</span></div>
          <div><strong>{row.diff_count}</strong><span>{t("diffs")}</span></div>
        </div>
        <p class="operator">{t("operator")}: {row.case.operator?.name || "—"}</p>
        <div class="actions">
          <button class="btn-sm" onclick={() => openCaseFolder(row.case_dir)}>{t("caseFolder")}</button>
        </div>
      </MacCard>
    {/each}
  {/if}
</div>

<style>
  .toolbar { margin-bottom: 12px; }
  .hint { margin: 0; font-size: 13px; color: var(--text-muted); }
  .empty-hint { font-size: 11px !important; color: var(--text-muted); }
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
