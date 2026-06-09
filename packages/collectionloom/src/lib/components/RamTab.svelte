<script>
import { invoke } from "../api/tauri.js";
import { defaultOutputPath } from "../api/portable.js";
import { guessPlatform } from "../window.js";
import GuideCard from "./GuideCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { ramCaptureGuide, appleVolatileDataGuide } from "../guides.js";
import { getLocale, subscribeLocale } from "../stores/locale.js";
let { sharedState, caseState = {}, busy, setBusy, setMsg, timeoutPromise } = $props();
let tools = $state([]);
let toolsLoading = $state(false);
let selectedTool = $state("");
let advancedOpen = $state(false);
let outputPath = $state("");
let locale = $state(getLocale());
let platform = $state(guessPlatform());

$effect(() => {
  defaultOutputPath("ram_capture.lime").then((p) => {
    if (!outputPath) outputPath = p;
  });
});
let compress = $state(true);
let progress = $state(null);
let ramSize = $state(null);

// Auto hash
let autoHash = $state(true);
let hashResult = $state("");

// Process list
let processList = $state([]);
let showProcesses = $state(false);
let ramCaptureTimeoutMs = $state(120000);

$effect(() => subscribeLocale((_, resolved) => {
  locale = resolved;
}));

const isMac = () => platform === "macos";

const text = {
  en: {
    title: "RAM Capture",
    subtitle: "Mode 1 is recommended; Mode 2 exposes advanced tool selection",
    recommended: "Recommended",
    advanced: "Mode 2: Advanced options",
    mode1: "Mode 1: Recommended",
    mode2: "Mode 2: Advanced",
    tool: "Tool:",
    output: "Output:",
    compress: "Compress",
    autoHash: "Auto hash after capture",
    capture: "▶ Capture RAM",
    detect: "Detecting tools…",
    selectTool: "— Select tool —",
    refresh: "Refresh",
    noTools: "No supported RAM capture tool found. Linux/Windows will prefer the recommended tool when available; macOS raw RAM capture is not supported.",
    listProcesses: "List Processes",
    runningProcesses: "Running Processes",
    refresh: "Refresh",
  },
  id: {
    title: "Tangkap RAM",
    subtitle: "Mode 1 direkomendasikan; Mode 2 membuka pemilihan alat lanjutan",
    recommended: "Rekomendasi",
    advanced: "Mode 2: Opsi lanjutan",
    mode1: "Mode 1: Rekomendasi",
    mode2: "Mode 2: Lanjutan",
    tool: "Alat:",
    output: "Keluaran:",
    compress: "Kompres",
    autoHash: "Hash otomatis setelah akuisisi",
    capture: "▶ Tangkap RAM",
    detect: "Mendeteksi alat…",
    selectTool: "— Pilih alat —",
    refresh: "Segarkan",
    noTools: "Tidak ada alat RAM capture yang didukung. Linux/Windows akan memilih alat rekomendasi bila tersedia; akuisisi RAM mentah di macOS tidak didukung.",
    listProcesses: "Daftar Proses",
    runningProcesses: "Proses Berjalan",
    refresh: "Segarkan",
  },
};
function tr(key) { return text[locale]?.[key] || text.en[key] || key; }

function toolLabel(tool) {
  if (tool === "Avml") return "AVML (recommended)";
  if (tool === "WinPmem") return "WinPmem (Windows)";
  if (tool === "LiME") return "LiME (manual / advanced)";
  return tool;
}

function computeRamCaptureTimeoutMs(bytes) {
  const base = 120000;
  if (!bytes || bytes <= 0) return base;
  const gib = bytes / (1024 ** 3);
  const scaled = Math.round(Math.max(base, gib * 45000));
  return Math.min(Math.max(scaled, base), 30 * 60 * 1000);
}

async function listTools() {
  toolsLoading = true;
  try {
    tools = await timeoutPromise(invoke("list_ram_tools"), 10000);
    if (tools.length) {
      selectedTool = tools.includes("Avml") ? "Avml" : tools[0];
    } else {
      selectedTool = "";
    }
    if (!tools.length) {
      setMsg(locale === "id" ? "PERINGATAN: Tidak ada alat RAM capture — tangkap dinonaktifkan sampai alat valid tersedia (lihat tab Prasyarat)" : "WARN: No RAM capture tools found — capture is disabled until a valid tool is available (see Prerequisites tab)");
    }
  } catch (e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`ERR: ${err}`);
  }
  try { ramSize = await timeoutPromise(invoke("get_ram_size"), 5000); } catch(e) {}
  ramCaptureTimeoutMs = computeRamCaptureTimeoutMs(ramSize);
  toolsLoading = false;
}
async function capture() {
  setBusy(true);
  hashResult = "";
  try {
    const storage = await timeoutPromise(
      invoke("verify_acquisition_storage", { output: outputPath }),
      10000
    );
    if (!storage.ok) {
      setMsg(`WARN: ${storage.notes}`);
      setBusy(false);
      return;
    }
    const result = await timeoutPromise(invoke("capture_ram", {
      tool: selectedTool,
      output: outputPath,
      compress,
      caseId: caseState.caseId || null,
      operator: caseState.operator || null,
    }), ramCaptureTimeoutMs);
    const sha256 = result?.sha256;
    const verified = result?.verified;
    setMsg(`OK: ${result?.message || "Capture complete"}`);
    if (autoHash && sha256) {
      hashResult = `SHA-256: ${sha256}${verified === false ? " (NOT verified — re-read mismatch)" : verified ? " (verified ×2)" : ""}`;
    } else if (autoHash && outputPath) {
      try {
        const hash = await timeoutPromise(invoke("hash_and_verify_evidence", { path: outputPath }), 30000);
        hashResult = `SHA-256: ${hash.sha256}${hash.verified ? " (verified ×2)" : " (NOT verified)"}`;
      } catch(e) {
        hashResult = `Hash failed: ${typeof e === "string" ? e : String(e)}`;
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

<div class="tab-content">
  <SectionHeader title={tr("title")} subtitle={tr("subtitle")} />
  {#if ramSize}<p class="info">System RAM: {(ramSize/1e9).toFixed(1)} GB</p>{/if}
  {#if !isMac()}
    <div class="recommendation">
      <PillBadge variant="active" label={tr("mode1")} />
      <span>{selectedTool ? toolLabel(selectedTool) : (locale === "id" ? "Memilih alat terbaik…" : "Selecting the best tool…")}</span>
    </div>
    {#if tools.length > 0}
      <p class="helper">
        {selectedTool === "Avml"
          ? (locale === "id"
              ? "AVML dipilih otomatis karena paling mudah dipakai lintas platform."
              : "AVML is selected automatically because it is the easiest cross-platform option.")
          : (locale === "id"
              ? "Gunakan opsi lanjutan hanya jika perlu workflow khusus."
              : "Use advanced options only if you need a special workflow.")}
      </p>
    {/if}
    <div class="row">
      <button class="btn-sm" onclick={() => (advancedOpen = !advancedOpen)} disabled={busy || toolsLoading}>
        {advancedOpen ? "−" : "+"} {advancedOpen ? tr("mode2") : tr("advanced")}
      </button>
      <button onclick={listTools} class="btn-sm" disabled={busy || toolsLoading}>{toolsLoading ? "…" : tr("refresh")}</button>
    </div>
    {#if advancedOpen}
      <div class="row">
        <label>{tr("tool")} <select bind:value={selectedTool} disabled={busy || toolsLoading}>
          <option value="">{toolsLoading ? tr("detect") : tr("selectTool")}</option>
          {#each tools as tool}<option value={tool}>{toolLabel(tool)}</option>{/each}
        </select></label>
      </div>
    {/if}
    {#if !toolsLoading && tools.length === 0}
      <p class="empty-hint">{tr("noTools")}</p>
    {/if}
    {#if selectedTool === "Avml"}
      <p class="helper">
        {locale === "id"
          ? "AVML di Linux biasanya butuh hak elevasi. Jika capture gagal, jalankan CollectionLoom dari shell sudo atau buka portable session dengan root."
          : "AVML on Linux usually needs elevated privileges. If capture fails, run CollectionLoom from a sudo shell or launch the portable session as root."}
      </p>
    {/if}
    <div class="row">
      <label>{tr("output")} <input type="text" bind:value={outputPath} disabled={busy} /></label>
      <label><input type="checkbox" bind:checked={compress} disabled={busy} /> {tr("compress")}</label>
    </div>
    <div class="row">
      <label><input type="checkbox" bind:checked={autoHash} disabled={busy} /> {tr("autoHash")}</label>
    </div>
    <div class="actions">
      <button onclick={capture} class="btn-primary" disabled={busy || toolsLoading || tools.length === 0}>{tr("capture")}</button>
      <button onclick={listProcesses} class="btn-sm" disabled={busy}>{tr("listProcesses")}</button>
    </div>
  {:else}
    <div class="apple-note">
      <PillBadge variant="warning" label={locale === "id" ? "Apple Volatile Data" : "Apple Volatile Data"} />
      <p>
        {locale === "id"
          ? "CollectionLoom tidak menyediakan raw RAM dump di macOS. Gunakan alur volatile data alternatif untuk Intel dan Apple Silicon."
          : "CollectionLoom does not provide raw RAM dumps on macOS. Use alternative volatile data workflows for Intel and Apple Silicon."}
      </p>
    </div>
  {/if}

  {#if hashResult}
  <div class="hash-result">{hashResult}</div>
  {/if}

  {#if showProcesses && processList.length > 0}
  <div class="process-section">
    <div class="process-header">
      <span>Running Processes ({processList.length})</span>
      <button onclick={refreshProcesses} class="btn-sm">{tr("refresh")}</button>
    </div>
    <div class="process-list">
      {#each processList as proc}
        <div class="process-item">{proc.pid} | {proc.name} | {(proc.memory_bytes / 1048576).toFixed(1)} MB | {proc.cpu_percent.toFixed(1)}% CPU</div>
      {/each}
    </div>
  </div>
  {/if}

  <GuideCard title={ramCaptureGuide.title} icon={ramCaptureGuide.icon} steps={ramCaptureGuide.steps} references={ramCaptureGuide.references} />
  {#if isMac()}
    <GuideCard title={appleVolatileDataGuide.title} icon={appleVolatileDataGuide.icon} steps={appleVolatileDataGuide.steps} references={appleVolatileDataGuide.references} />
  {/if}
</div>

<style>
.info { font-size:12px; color:var(--text-secondary); margin-bottom:10px; }
.recommendation { display:flex; gap:10px; align-items:center; margin-bottom:8px; color:var(--text-secondary); font-size:12px; }
.helper { margin:0 0 12px; font-size:12px; color:var(--text-muted); line-height:1.5; }
.apple-note { display:flex; flex-direction:column; gap:8px; margin:12px 0; padding:12px; border:1px solid var(--border); border-radius:8px; background:var(--input-bg); color:var(--text-secondary); font-size:12px; line-height:1.5; }
.empty-hint { font-size:12px; color:var(--text-muted); margin:-4px 0 12px; }
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
