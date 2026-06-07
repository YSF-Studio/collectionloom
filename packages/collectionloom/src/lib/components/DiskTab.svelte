<script>
import { invoke, openDialog, isPreviewError, listenEvent, isTauri } from "../api/tauri.js";
import { defaultOutputPath } from "../api/portable.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import FormatPicker from "./ui/FormatPicker.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import ConfirmDialog from "./ui/ConfirmDialog.svelte";
import { diskImagingGuide } from "../guides.js";
import { wbPillLabel } from "../wb.js";
import { getLocale, subscribeLocale } from "../stores/locale.js";

let {
  sharedState,
  caseState = {},
  wbDevice = "",
  busy,
  setBusy,
  setMsg,
  timeoutPromise,
  onProgressChange = () => {},
  onDeviceSelect = () => {},
} = $props();

let disks = $state([]);
let disksLoading = $state(false);
let showConfirmStart = $state(false);
let showConfirmDisableWb = $state(false);
let selectedDisk = $state("");
let sourceMode = $state("all");
let destPath = $state("");

$effect(() => {
  defaultOutputPath("image.dd").then((p) => {
    if (!destPath) destPath = p;
  });
});
let splitSize = $state("0");
let shouldVerify = $state(true);
let imageFormat = $state("raw");
let hashMd5 = $state(true);
let hashSha256 = $state(true);
let progress = $state({ percent: 0, status: "Idle", bytesProcessed: 0, totalBytes: 0, isDone: false, error: null });
let collBusy = $state(false);
let pollId = $state(null);
let eta = $state("");
let startTime = $state(null);
let bitlockerDetected = $state(false);
let encryptionScan = $state(null);
let wbStatus = $state(null);
let hpaReport = $state(null);
let hpaBusy = $state(false);
let imagingSummary = $state(null);
let unlistenComplete = null;
let unlistenError = null;
let locale = $state(getLocale());

$effect(() => subscribeLocale((_, resolved) => {
  locale = resolved;
}));

const text = {
  en: {
    acquireDrive: "Acquire Drive",
    sourceDrive: "Source Drive",
    formatHashing: "Format & Hashing",
    destination: "Destination",
    noDisks: "WARN: No disks detected — connect a drive and click Refresh",
    noDisksEmpty: "No disks detected",
    noDisksHint: "Connect a source device and click Refresh.",
    selectDisk: "WARN: Select a disk first",
    writeBlockerOk: "OK: Software write-blocker enabled",
    writeBlockerWarn: "WARN: Write-blocker not confirmed",
    hpaWarn: "WARN: HPA/DCO anomaly detected — review report",
    hpaOk: "OK: No HPA/DCO anomalies",
    hpaInfo: "INFO: HPA/DCO not supported on this device",
    diskFirst: "WARN: Select a disk first",
    refresh: "Refresh",
    sourceOnly: "All sources",
    physicalOnly: "Physical only",
    logicalOnly: "Logical only",
    selectFromList: "Select a disk from the list beside this panel.",
    startAcq: "Start Acquisition",
    noDisk: "No disks detected",
    wbLabel: "Write-Blocker:",
    wbEnable: "Enable Software Write-Blocker",
    wbDisable: "Disable",
    checkHpa: "Check HPA/DCO",
    sourcePhysical: "Physical source",
    sourceLogical: "Logical source",
    ssdRisk: "SSD — TRIM risk",
    encDetected: "Encryption Detected",
    ssdWarn: "SSD TRIM may have erased deleted data. Use a hardware write blocker when possible.",
    acqSummary: "Acquisition Summary",
    sectorsRead: "Sectors read",
    duration: "Duration",
    avgSpeed: "Avg speed",
    errorSectors: "Error sectors",
    zeroed: "zeroed in image",
    sourceIntegrity: "Source integrity",
    badSectorLog: "Bad-sector log",
    startDiskAcq: "Start Disk Acquisition?",
    startDiskMsg: "This will begin sector-by-sector imaging of the selected drive. Ensure write-blocker is active and the destination has sufficient free space.",
    disableWbTitle: "Disable Write-Blocker?",
    disableWbMsg: "Disabling the software write-blocker allows writes to the source drive. Only proceed if imaging is complete and you intend to modify the device.",
    disableWbConfirm: "Disable Write-Blocker",
  },
  id: {
    acquireDrive: "Akuisisi Drive",
    sourceDrive: "Drive Sumber",
    formatHashing: "Format & Hashing",
    destination: "Tujuan",
    noDisks: "PERINGATAN: Tidak ada disk terdeteksi — sambungkan drive lalu klik Segarkan",
    selectDisk: "PERINGATAN: Pilih disk dulu",
    writeBlockerOk: "OK: Software write-blocker aktif",
    writeBlockerWarn: "PERINGATAN: Write-blocker belum terkonfirmasi",
    hpaWarn: "PERINGATAN: Anomali HPA/DCO terdeteksi — tinjau laporan",
    hpaOk: "OK: Tidak ada anomali HPA/DCO",
    hpaInfo: "INFO: HPA/DCO tidak didukung pada perangkat ini",
    diskFirst: "PERINGATAN: Pilih disk dulu",
    refresh: "Segarkan",
    sourceOnly: "Semua sumber",
    physicalOnly: "Hanya fisik",
    logicalOnly: "Hanya logis",
    selectFromList: "Pilih disk dari daftar di samping panel ini.",
    startAcq: "Mulai Akuisisi",
    noDisk: "Tidak ada disk terdeteksi",
    wbLabel: "Write-Blocker:",
    wbEnable: "Aktifkan Software Write-Blocker",
    wbDisable: "Nonaktifkan",
    checkHpa: "Periksa HPA/DCO",
    sourcePhysical: "Sumber fisik",
    sourceLogical: "Sumber logis",
    ssdRisk: "SSD — risiko TRIM",
    encDetected: "Enkripsi Terdeteksi",
    ssdWarn: "TRIM SSD mungkin telah menghapus data terhapus. Gunakan hardware write blocker bila memungkinkan.",
    acqSummary: "Ringkasan Akuisisi",
    sectorsRead: "Sektor terbaca",
    duration: "Durasi",
    avgSpeed: "Kecepatan rata-rata",
    errorSectors: "Sektor error",
    zeroed: "di-nol-kan di image",
    sourceIntegrity: "Integritas sumber",
    badSectorLog: "Log sektor rusak",
    startDiskAcq: "Mulai Akuisisi Disk?",
    startDiskMsg: "Ini akan memulai imaging sektor-per-sektor dari drive yang dipilih. Pastikan write-blocker aktif dan tujuan memiliki ruang yang cukup.",
    disableWbTitle: "Nonaktifkan Write-Blocker?",
    disableWbMsg: "Menonaktifkan software write-blocker akan memungkinkan penulisan ke drive sumber. Lanjutkan hanya jika imaging sudah selesai dan Anda memang ingin memodifikasi perangkat.",
    disableWbConfirm: "Nonaktifkan Write-Blocker",
  },
};
function tr(key) { return text[locale]?.[key] || text.en[key] || key; }

function finishImagingPoll(p) {
  if (pollId) clearInterval(pollId);
  pollId = null;
  collBusy = false;
  eta = "";
  startTime = null;
  progress = p ?? progress;
  if (p?.summary) imagingSummary = p.summary;
  setMsg(p?.error ? `ERR: ${p.error}` : imagingSummary?.errorSectors > 0
    ? `WARN: Imaging complete with ${imagingSummary.errorSectors} bad sector(s) zero-filled — see log`
    : "OK: Imaging complete");
  onProgressChange({ progress: p ?? progress, collBusy: false, eta: "", selectedDisk, imageFormat });
}

function onImagingComplete(payload) {
  if (payload?.summary) imagingSummary = payload.summary;
  else if (payload?.hash && !imagingSummary) {
    imagingSummary = { sha256: payload.hash };
  }
  finishImagingPoll({ ...progress, isDone: true, error: null, summary: imagingSummary });
}

$effect(() => {
  if (!isTauri()) return;
  listenEvent("imaging_complete", (event) => onImagingComplete(event.payload)).then((fn) => {
    unlistenComplete = fn;
  });
  listenEvent("imaging_error", (event) => {
    finishImagingPoll({ ...progress, isDone: true, error: String(event.payload ?? "Imaging failed") });
    setMsg(`ERR: ${event.payload}`);
  }).then((fn) => {
    unlistenError = fn;
  });
  return () => {
    unlistenComplete?.();
    unlistenError?.();
  };
});

let visibleDisks = $derived(
  disks.filter((disk) => sourceMode === "all" || disk.sourceKind === sourceMode)
);
let selectedDiskInfo = $derived(visibleDisks.find((d) => d.device === selectedDisk) || null);

$effect(() => {
  sharedState.progress = progress;
  sharedState.collBusy = collBusy;
  sharedState.eta = eta;
  sharedState.selectedDisk = selectedDisk;
  onProgressChange({ progress, collBusy, eta, selectedDisk, imageFormat });
});

async function listDisks() {
  disksLoading = true;
  try {
    disks = await timeoutPromise(invoke("list_disks"), 15000);
    if (selectedDisk && !disks.some((d) => d.device === selectedDisk)) {
      selectedDisk = "";
    }
    if (!disks.length) setMsg(tr("noDisks"));
  } catch (e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT" && !isPreviewError(e)) setMsg(`ERR: ${err}`);
  }
  disksLoading = false;
}

function selectDisk(disk) {
  selectedDisk = disk.device;
  onDeviceSelect({
    device: disk.device,
    wbActive: wbStatus?.active ?? wbStatus?.enabled ?? false,
  });
  checkEncryption();
  checkWriteBlocker();
}

function sourceModeLabel(mode) {
  if (mode === "physical") return tr("physicalOnly");
  if (mode === "logical") return tr("logicalOnly");
  return tr("sourceOnly");
}

async function checkEncryption() {
  bitlockerDetected = false;
  if (!selectedDisk) return;
  try {
    encryptionScan = await timeoutPromise(invoke("scan_encryption"), 15000);
    const drives = encryptionScan?.drives || encryptionScan?.volumes || [];
    bitlockerDetected = drives.some(
      (d) =>
        (d.device === selectedDisk || d.path === selectedDisk) &&
        (d.encrypted || d.type?.toLowerCase?.().includes("bitlocker"))
    );
  } catch {
    /* best effort */
  }
}

async function checkWriteBlocker() {
  if (!selectedDisk) {
    wbStatus = null;
    return;
  }
  try {
    wbStatus = await invoke("check_write_blocker", { device: selectedDisk });
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDisk, wbActive: active });
  } catch {
    wbStatus = null;
    onDeviceSelect({ device: selectedDisk, wbActive: false });
  }
}

async function enableWriteBlocker() {
    if (!selectedDisk) {
    setMsg(tr("selectDisk"));
    return;
  }
  setBusy(true);
  try {
    wbStatus = await timeoutPromise(invoke("enable_write_blocker", { device: selectedDisk }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDisk, wbActive: active });
    setMsg(active ? tr("writeBlockerOk") : tr("writeBlockerWarn"));
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function detectHpaDco() {
  if (!selectedDisk) {
    setMsg(tr("selectDisk"));
    return;
  }
  hpaBusy = true;
  hpaReport = null;
  try {
    hpaReport = await timeoutPromise(invoke("hpa_dco_detect", { device: selectedDisk }), 30000);
    if (hpaReport?.hpaDetected || hpaReport?.dcoDetected) {
      setMsg(tr("hpaWarn"));
    } else {
      setMsg(hpaReport?.supported ? tr("hpaOk") : tr("hpaInfo"));
    }
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  hpaBusy = false;
}

function requestDisableWriteBlocker() {
  showConfirmDisableWb = true;
}

async function disableWriteBlocker() {
  showConfirmDisableWb = false;
  if (!selectedDisk) return;
  setBusy(true);
  try {
    wbStatus = await timeoutPromise(invoke("disable_write_blocker", { device: selectedDisk }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDisk, wbActive: active });
    setMsg("OK: Software write-blocker disabled");
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

$effect(() => {
  if (selectedDisk) {
    checkEncryption();
    checkWriteBlocker();
  }
});

$effect(() => {
  if (wbDevice && wbDevice !== selectedDisk) {
    selectedDisk = wbDevice;
  }
});

async function browseDestination() {
  const picked = await openDialog({ directory: false, multiple: false });
  if (picked) destPath = picked;
}

function resolveDestPath() {
  if (imageFormat === "e01") {
    return destPath.replace(/\.(dd|raw|aff4)?$/i, "") + ".E01";
  }
  if (imageFormat === "aff4") {
    return destPath.replace(/\.(dd|raw|e01)?$/i, "") + ".aff4";
  }
  return destPath;
}

function requestStartImaging() {
  if (!selectedDisk || !destPath) {
    setMsg(locale === "id" ? "PERINGATAN: Pilih disk dan tujuan" : "WARN: Select a disk and destination");
    return;
  }
  showConfirmStart = true;
}

async function startImaging() {
  showConfirmStart = false;
  if (!selectedDisk || !destPath) return;
  collBusy = true;
  startTime = Date.now();
  imagingSummary = null;
  const destination = resolveDestPath();
  try {
    await timeoutPromise(
      invoke("start_disk_imaging", {
        source: selectedDisk,
        destination,
        splitSizeMb: parseInt(splitSize) || 0,
        verify: shouldVerify || hashSha256,
        imageFormat,
        caseId: caseState.caseId || null,
        operator: caseState.operator || null,
      }),
      5000
    );
    pollId = setInterval(async () => {
      try {
        const p = await invoke("get_imaging_progress");
        progress = p;
        if (p.bytesProcessed > 0 && startTime) {
          const elapsed = (Date.now() - startTime) / 1000;
          const speed = p.bytesProcessed / elapsed;
          if (speed > 0 && p.totalBytes > 0) {
            const remaining = (p.totalBytes - p.bytesProcessed) / speed;
            if (remaining < 60) eta = `${Math.round(remaining)}s`;
            else if (remaining < 3600) eta = `${Math.round(remaining / 60)}m ${Math.round(remaining % 60)}s`;
            else eta = `${Math.round(remaining / 3600)}h ${Math.round((remaining % 3600) / 60)}m`;
          }
        }
        onProgressChange({ progress: p, collBusy: true, eta, selectedDisk, imageFormat });
        if (p.isDone) {
          finishImagingPoll(p);
        }
      } catch {
        clearInterval(pollId);
        collBusy = false;
        eta = "";
        startTime = null;
      }
    }, 500);
  } catch (e) {
    collBusy = false;
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`ERR: ${err}`);
  }
}

async function cancelImaging() {
  await invoke("cancel_imaging");
  if (pollId) clearInterval(pollId);
  collBusy = false;
  progress = { ...progress, status: "Cancelled" };
  eta = "";
  startTime = null;
}

$effect(() => {
  listDisks();
});
</script>

<div class="tab-content disk-tab">
  <SectionHeader title={tr("acquireDrive")} subtitle={locale === "id" ? "Disk Imaging — akuisisi sektor-per-sektor dari disk sumber atau volume logis dengan verifikasi hash" : "Disk Imaging — sector-by-sector acquisition of a source disk or logical volume with hash verification"} />

  <MacCard title={tr("sourceDrive")}>
    <div class="disk-layout">
      <aside class="disk-sidebar" aria-label="Available disks">
        <div class="disk-sidebar-head">
          <span class="disk-sidebar-title">{locale === "id" ? "Daftar sumber" : "Source list"}</span>
          <select bind:value={sourceMode} class="source-mode" aria-label="Filter source type" disabled={collBusy || disksLoading}>
            <option value="all">{tr("sourceOnly")}</option>
            <option value="physical">{tr("physicalOnly")}</option>
            <option value="logical">{tr("logicalOnly")}</option>
          </select>
          <button onclick={listDisks} class="btn-sm" disabled={collBusy || disksLoading}>
            {#if disksLoading}<span class="spinner">↻</span>{/if}
            {disksLoading ? "…" : tr("refresh")}
          </button>
        </div>
        {#if disksLoading}
          <p class="disk-sidebar-msg">{locale === "id" ? "Memuat disk…" : "Loading disks…"}</p>
        {:else if visibleDisks.length === 0}
          <div class="empty-state compact">
            <p>{tr("noDisk")}</p>
            <p class="empty-hint">{sourceModeLabel(sourceMode)} {locale === "id" ? "kosong." : "is empty."} {tr("noDisksHint")}</p>
          </div>
        {:else}
          <ul class="disk-list">
            {#each visibleDisks as disk}
              <li>
                <button
                  type="button"
                  class="disk-item"
                  class:selected={selectedDisk === disk.device}
                  disabled={collBusy || busy}
                  onclick={() => selectDisk(disk)}
                >
                  <span class="disk-item-device">{disk.device}</span>
                  <span class="disk-item-model">{disk.model || "Unknown"}</span>
                  <span class="disk-item-meta">
                    {(disk.sizeBytes / 1e9).toFixed(1)} GB · {disk.isSsd ? "SSD" : "HDD"} · {disk.sourceKind === "logical" ? (locale === "id" ? "Logis" : "Logical") : (locale === "id" ? "Fisik" : "Physical")}
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </aside>

      <div class="disk-main">
        {#if !selectedDiskInfo}
          <p class="disk-prompt">{tr("selectFromList")}</p>
        {:else}
          <div class="drive-detail">
            <span>{selectedDiskInfo.device}</span>
            <span>{selectedDiskInfo.model || (locale === "id" ? "Model tidak diketahui" : "Unknown model")}</span>
            <span>{(selectedDiskInfo.sizeBytes / 1e9).toFixed(1)} GB</span>
            <PillBadge variant={selectedDiskInfo.sourceKind === "logical" ? "active" : "warning"} label={selectedDiskInfo.sourceKind === "logical" ? tr("sourceLogical") : tr("sourcePhysical")} />
            {#if selectedDiskInfo.isSsd}<PillBadge variant="warning" label={tr("ssdRisk")} />{/if}
            {#if bitlockerDetected}<PillBadge variant="warning" label={tr("encDetected")} />{/if}
          </div>
          {#if selectedDiskInfo.isSsd}
            <p class="warn-text">{tr("ssdWarn")}</p>
          {/if}
          <div class="wb-section">
            {#if wbStatus}
              <div class="wb-row">
                <span class="wb-label">{tr("wbLabel")}</span>
                <PillBadge
                  variant={(wbStatus.active ?? wbStatus.enabled) ? "active" : "inactive"}
                  label={wbPillLabel(wbStatus)}
                />
              </div>
            {/if}
            <div class="wb-btns">
              <button onclick={enableWriteBlocker} class="btn-sm" disabled={collBusy || busy || !selectedDisk}>{tr("wbEnable")}</button>
              <button onclick={requestDisableWriteBlocker} class="btn-sm" disabled={collBusy || busy || !selectedDisk || !wbStatus?.software}>{tr("wbDisable")}</button>
              <button onclick={checkWriteBlocker} class="btn-sm" disabled={collBusy || !selectedDisk}>{tr("refresh")}</button>
              <button onclick={detectHpaDco} class="btn-sm" disabled={collBusy || busy || hpaBusy || !selectedDisk}>
                {hpaBusy ? (locale === "id" ? "Memeriksa HPA/DCO…" : "Checking HPA/DCO…") : tr("checkHpa")}
              </button>
            </div>
            {#if hpaReport}
              <div class="hpa-report">
                {#if hpaReport.hpaDetected}<PillBadge variant="warning" label={locale === "id" ? "HPA Terdeteksi" : "HPA Detected"} />{/if}
                {#if hpaReport.dcoDetected}<PillBadge variant="warning" label={locale === "id" ? "DCO Terdeteksi" : "DCO Detected"} />{/if}
                {#if hpaReport.hiddenSectors != null}<span class="wb-detail">{locale === "id" ? "Sektor tersembunyi" : "Hidden sectors"}: {hpaReport.hiddenSectors}</span>{/if}
                <span class="wb-detail">{hpaReport.notes}</span>
              </div>
            {/if}
            {#if wbStatus?.notes}<p class="wb-notes">{wbStatus.notes}</p>{/if}
          </div>
        {/if}
      </div>
    </div>
  </MacCard>

  <MacCard title={tr("formatHashing")}>
    <FormatPicker bind:format={imageFormat} bind:hashMd5 bind:hashSha256 disabled={collBusy} />
      <div class="split-row">
      <label>{locale === "id" ? "Pecah (MB):" : "Split (MB):"} <input type="number" bind:value={splitSize} disabled={collBusy} placeholder={locale === "id" ? "0 = tanpa pecah" : "0 = no split"} /></label>
      <label class="check"><input type="checkbox" bind:checked={shouldVerify} disabled={collBusy} /> {locale === "id" ? "Verifikasi setelah penulisan" : "Verify after write"}</label>
    </div>
  </MacCard>

  <MacCard title={tr("destination")}>
    <div class="row">
      <input type="text" bind:value={destPath} disabled={collBusy} class="full" placeholder="/path/to/image.dd" />
      <button onclick={browseDestination} class="btn-sm" disabled={collBusy}>{locale === "id" ? "Jelajah" : "Browse"}</button>
    </div>
  </MacCard>

  <div class="actions">
    {#if !collBusy}
      <button onclick={requestStartImaging} class="btn-primary" disabled={!selectedDisk}>{tr("startAcq")}</button>
    {:else}
      <button onclick={cancelImaging} class="btn-danger">{locale === "id" ? "Hentikan" : "Stop"}</button>
    {/if}
  </div>

  {#if imagingSummary}
    <MacCard title={tr("acqSummary")}>
      <div class="summary-grid">
        <span class="summary-item">{tr("sectorsRead")}: {imagingSummary.sectorsRead?.toLocaleString?.() ?? imagingSummary.sectorsRead ?? "—"}</span>
        <span class="summary-item">{tr("duration")}: {imagingSummary.durationSecs?.toFixed?.(1) ?? imagingSummary.durationSecs ?? "—"}s</span>
        <span class="summary-item">{tr("avgSpeed")}: {((imagingSummary.avgSpeedBytesPerSec ?? 0) / 1e6).toFixed(1)} MB/s</span>
        <span class:warn-sector={(imagingSummary.errorSectors ?? 0) > 0}>
          {tr("errorSectors")}: {imagingSummary.errorSectors ?? 0}
          {#if (imagingSummary.errorSectors ?? 0) > 0} ({tr("zeroed")}){/if}
        </span>
        <span>{tr("sourceIntegrity")}: {imagingSummary.sourceIntegrityOk ? "OK" : "FAILED"}</span>
        <span class="mono">SHA-256: {imagingSummary.sha256 ?? "—"}</span>
        {#if imagingSummary.badSectorsLog}
          <span class="mono">{tr("badSectorLog")}: {imagingSummary.badSectorsLog}</span>
        {/if}
      </div>
    </MacCard>
  {/if}

  <GuideCard title={diskImagingGuide.title} icon={diskImagingGuide.icon} steps={diskImagingGuide.steps} references={diskImagingGuide.references} />
</div>

<ConfirmDialog
  open={showConfirmStart}
  title={tr("startDiskAcq")}
  message={tr("startDiskMsg")}
  confirmLabel={tr("startAcq")}
  variant="primary"
  onConfirm={startImaging}
  onCancel={() => (showConfirmStart = false)}
/>

<ConfirmDialog
  open={showConfirmDisableWb}
  title={tr("disableWbTitle")}
  message={tr("disableWbMsg")}
  confirmLabel={tr("disableWbConfirm")}
  variant="danger"
  onConfirm={disableWriteBlocker}
  onCancel={() => (showConfirmDisableWb = false)}
/>

<style>
  .disk-layout {
    display: grid;
    grid-template-columns: minmax(220px, 280px) 1fr;
    gap: 14px;
    align-items: start;
  }
  .disk-sidebar {
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--input-bg);
    overflow: hidden;
    max-height: 320px;
    display: flex;
    flex-direction: column;
  }
  .disk-sidebar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    background: var(--btn-secondary-bg);
  }
  .source-mode {
    min-width: 120px;
    font-size: 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--input-bg);
    color: var(--text);
    padding: 6px 8px;
  }
  .disk-sidebar-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .disk-sidebar-msg {
    margin: 0;
    padding: 12px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .disk-list {
    list-style: none;
    margin: 0;
    padding: 6px;
    overflow-y: auto;
    flex: 1;
  }
  .disk-item {
    width: 100%;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    cursor: pointer;
  }
  .disk-item:hover:not(:disabled) {
    background: var(--btn-secondary-bg);
  }
  .disk-item.selected {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 12%, transparent);
  }
  .disk-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .disk-item-device {
    font-size: 12px;
    font-weight: 600;
    font-family: var(--mono);
  }
  .disk-item-model {
    font-size: 12px;
    color: var(--text-secondary);
  }
  .disk-item-meta {
    font-size: 11px;
    color: var(--text-muted);
  }
  .disk-main {
    min-width: 0;
  }
  .disk-prompt {
    margin: 0;
    padding: 16px 4px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .empty-state.compact {
    padding: 12px;
  }
  .empty-state.compact p {
    margin: 0 0 4px;
    font-size: 12px;
  }
  .row { display: flex; gap: 8px; align-items: center; }
  .full { flex: 1; }
  .row input {
    background: var(--input-bg); color: var(--text); border: 1px solid var(--border);
    border-radius: 8px; padding: 8px 12px; font-size: 13px;
  }
  .drive-detail { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; font-size: 12px; color: var(--text-secondary); }
  .warn-text { margin: 0; font-size: 12px; color: var(--warn); }
  .wb-section { margin-top: 10px; display: flex; flex-direction: column; gap: 8px; }
  .wb-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .wb-label { font-size: 11px; color: var(--text-muted); font-weight: 600; }
  .wb-detail { font-size: 11px; color: var(--text-secondary); }
  .empty-hint { font-size: 11px !important; color: var(--text-muted); }
  .wb-btns { display: flex; gap: 8px; flex-wrap: wrap; }
  .hpa-report { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary);
    overflow: visible;
  }
  .summary-item { white-space: normal; word-break: break-word; overflow: visible; }
  .warn-sector { color: var(--warn); font-weight: 600; }
  .wb-notes { margin: 0; font-size: 11px; color: var(--text-muted); }
  .split-row { display: flex; gap: 16px; flex-wrap: wrap; font-size: 13px; align-items: center; }
  .check { display: flex; align-items: center; gap: 6px; }
  .actions { display: flex; gap: 10px; margin: 8px 0 16px; }
  .btn-primary { padding: 10px 28px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-danger { padding: 10px 28px; background: var(--danger); color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; }
  .btn-sm { padding: 8px 14px; background: var(--btn-secondary-bg); color: var(--btn-secondary-text); border: 1px solid var(--border); border-radius: 8px; cursor: pointer; font-size: 12px; }
  .mono { margin: 0; font-size: 11px; white-space: pre-wrap; word-break: break-all; font-family: var(--mono); color: var(--text-secondary); }
</style>
