<script>
import { invoke } from "../api/tauri.js";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { getResolvedLocale, subscribeLocale } from "../stores/locale.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();

let report = $state(null);
let loading = $state(false);
let locale = $state(getResolvedLocale());

const categoryLabels = {
  en: {
    pure_rust: "Pure Rust (built-in)",
    system_library: "System library",
    external_binary: "External binary",
    privilege: "OS privilege",
    portable: "Portable Kit",
    summary: "Summary",
    ready: "Ready",
    actionNeeded: "Action needed",
    runningChecks: "Running checks",
    scanText: "Scanning kit ./tools/, app resources, PATH, libraries, and privileges…",
    reRun: "Re-run Pre-flight Check",
    downloadable: "Downloadable tools",
    embedded: "Embedded tools",
    sourceBuilt: "Source-built app",
    portableStorage: "Portable storage",
    noKit:
      "No kit ./tools/ and no embedded tools — run npm run download-tools before build, or copy source-specific tools beside the portable app for USB kits. Actions that need those tools will stay disabled.",
    actionNote:
      "Downloadable tools can be embedded at build time (npm run download-tools) or placed in ./tools/ on forensic USB kits. Source-specific tools may still require manual staging, and related buttons stay disabled until the tool is present.",
    missing: "Missing",
    ok: "OK",
    usedFor: "Used for",
    manifestLoaded: "manifest loaded",
  },
  id: {
    pure_rust: "Pure Rust (bawaan)",
    system_library: "Pustaka sistem",
    external_binary: "Binary eksternal",
    privilege: "Hak OS",
    portable: "Portable Kit",
    summary: "Ringkasan",
    ready: "Siap",
    actionNeeded: "Perlu tindakan",
    runningChecks: "Menjalankan pengecekan",
    scanText: "Memindai kit ./tools/, resource aplikasi, PATH, pustaka, dan hak akses…",
    reRun: "Jalankan Ulang Pre-flight Check",
    downloadable: "Tool yang bisa diunduh",
    embedded: "Tool yang tertanam",
    sourceBuilt: "Aplikasi hasil source build",
    portableStorage: "Penyimpanan portable",
    noKit:
      "Tidak ada kit ./tools/ dan tidak ada tool tertanam — jalankan npm run download-tools sebelum build, atau salin tool source-specific di sebelah aplikasi portable untuk USB kit. Aksi yang membutuhkan tool itu akan tetap nonaktif.",
    actionNote:
      "Tool yang bisa diunduh dapat dibundel saat build (npm run download-tools) atau ditempatkan di ./tools/ pada USB kit forensik. Tool source-specific mungkin tetap perlu staging manual, dan tombol terkait akan tetap nonaktif sampai tool tersedia.",
    missing: "Hilang",
    ok: "OK",
    usedFor: "Digunakan untuk",
    manifestLoaded: "manifest termuat",
  },
};

const t = (key) => categoryLabels[locale]?.[key] ?? categoryLabels.en[key] ?? key;

const unsubscribe = subscribeLocale((_, resolved) => {
  locale = resolved;
});

$effect(() => () => unsubscribe());

async function runCheck() {
  loading = true;
  setBusy(true);
  try {
    report = await timeoutPromise(invoke("run_preflight_check"), 15000);
    if (report?.missingCount > 0) {
      setMsg(`WARN: ${report.missingCount} tool(s) missing — some capture actions will stay disabled until they are available`);
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
    title={locale === "id" ? "Prasyarat" : "Prerequisites"}
    subtitle={locale === "id" ? "Periksa apa yang siap sekarang: tool bundel, tool source-specific, pustaka, dan hak akses" : "Check what is ready now: bundled tools, source-specific tools, libraries, and privileges"}
  />

  <div class="toolbar">
    <button class="btn-sm" onclick={runCheck} disabled={busy || loading}>
      {#if loading}<span class="spinner">↻</span>{/if}
      {loading ? (locale === "id" ? "Memeriksa…" : "Checking…") : t("reRun")}
    </button>
  </div>

  {#if loading && !report}
    <MacCard title={t("runningChecks")}>
      <p class="hint">{t("scanText")}</p>
    </MacCard>
  {:else if report}
    <MacCard title={t("portable")}>
      <div class="summary-row">
        <PillBadge
          variant={report.portable?.distributionMode === "portable" ? "active" : "info"}
          label={report.portable?.distributionMode === "portable" ? t("portable") : t("sourceBuilt")}
        />
        {#if report.portable?.portableMode && report.portable?.distributionMode !== "portable"}
          <PillBadge variant="warning" label={t("portableStorage")} />
        {/if}
        {#if report.portable?.kitRoot}
          <span class="platform kit-path" title={report.portable.kitRoot}>Kit: {report.portable.kitRoot}</span>
        {/if}
      </div>
      <p class="summary-text">
          {#if report.portable?.toolsDirExists}
          {t("downloadable")}: <code>{report.portable.toolsDir}</code>
          {#if report.portable.manifestLoaded} · {t("manifestLoaded")}{/if}
        {:else if report.portable?.bundledToolsAvailable}
          {t("embedded")}: <code>{report.portable.bundledToolsDir}</code>
          {#if report.portable.manifestLoaded} · {t("manifestLoaded")}{/if}
        {:else}
          {t("noKit")} <code>./tools/</code>
        {/if}
      </p>
    </MacCard>

    <MacCard title={t("summary")}>
      <div class="summary-row">
        <PillBadge
          variant={report.missingCount === 0 && report.warningCount === 0 ? "active" : "warning"}
          label={report.missingCount === 0 && report.warningCount === 0 ? t("ready") : t("actionNeeded")}
        />
          <span class="platform">{report.platform} · {report.checkedAt?.slice(0, 19).replace("T", " ")}</span>
      </div>
      <p class="summary-text">{report.summary}</p>
      {#if report.missingCount > 0}
        <p class="warn-note">
          {t("actionNote")} <code>npm run download-tools</code> <code>./tools/</code>
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
                  label={item.available ? t("ok") : t("missing")}
                />
                <strong>{item.name}</strong>
                <span class="for">{t("usedFor")} {item.requiredFor}</span>
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
