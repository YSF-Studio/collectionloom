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

let selectedDiskInfo = $derived(disks.find((d) => d.device === selectedDisk) || null);

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
    if (!disks.length) setMsg("WARN: No disks detected — connect a drive and click Refresh");
  } catch (e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT" && !isPreviewError(e)) setMsg(`ERR: ${err}`);
  }
  disksLoading = false;
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
    setMsg("WARN: Select a disk first");
    return;
  }
  setBusy(true);
  try {
    wbStatus = await timeoutPromise(invoke("enable_write_blocker", { device: selectedDisk }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDisk, wbActive: active });
    setMsg(active ? "OK: Software write-blocker enabled" : "WARN: Write-blocker not confirmed");
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function detectHpaDco() {
  if (!selectedDisk) {
    setMsg("WARN: Select a disk first");
    return;
  }
  hpaBusy = true;
  hpaReport = null;
  try {
    hpaReport = await timeoutPromise(invoke("hpa_dco_detect", { device: selectedDisk }), 30000);
    if (hpaReport?.hpaDetected || hpaReport?.dcoDetected) {
      setMsg("WARN: HPA/DCO anomaly detected — review report");
    } else {
      setMsg(hpaReport?.supported ? "OK: No HPA/DCO anomalies" : "INFO: HPA/DCO not supported on this device");
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
    setMsg("WARN: Select a disk and destination");
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
  <SectionHeader title="Acquire Drive" subtitle="Disk Imaging — sector-by-sector acquisition with hash verification" />

  <MacCard title="Source Drive">
    <div class="row">
      <select
        bind:value={selectedDisk}
        disabled={collBusy || disksLoading}
        class="full"
        onchange={() => onDeviceSelect({ device: selectedDisk, wbActive: wbStatus?.active ?? wbStatus?.enabled ?? false })}
      >
        <option value="">{disksLoading ? "Loading disks…" : "— Select disk —"}</option>
        {#each disks as disk}
          <option value={disk.device}>
            {disk.device} — {disk.model || "Unknown"} ({(disk.sizeBytes / 1e9).toFixed(1)} GB {disk.isSsd ? "SSD" : "HDD"})
          </option>
        {/each}
      </select>
      <button onclick={listDisks} class="btn-sm" disabled={collBusy || disksLoading}>
        {#if disksLoading}<span class="spinner">↻</span>{/if}
        {disksLoading ? "Loading…" : "Refresh"}
      </button>
    </div>
    {#if !disksLoading && disks.length === 0}
      <div class="empty-state">
        <span class="icon">💾</span>
        <p>No disks detected</p>
        <p class="empty-hint">Connect a source drive and click Refresh. All physical disks are listed via native diskutil / PowerShell.</p>
      </div>
    {/if}
    {#if selectedDiskInfo}
      <div class="drive-detail">
        <span>{selectedDiskInfo.device}</span>
        <span>{selectedDiskInfo.model || "Unknown model"}</span>
        <span>{(selectedDiskInfo.sizeBytes / 1e9).toFixed(1)} GB</span>
        {#if selectedDiskInfo.isSsd}<PillBadge variant="warning" label="SSD — TRIM risk" />{/if}
        {#if bitlockerDetected}<PillBadge variant="warning" label="Encryption Detected" />{/if}
      </div>
    {/if}
    {#if selectedDiskInfo?.isSsd}
      <p class="warn-text">SSD TRIM may have erased deleted data. Use a hardware write blocker when possible.</p>
    {/if}
    {#if selectedDisk}
      <div class="wb-section">
        {#if wbStatus}
          <div class="wb-row">
            <span class="wb-label">Write-Blocker:</span>
            <PillBadge
              variant={(wbStatus.active ?? wbStatus.enabled) ? "active" : "inactive"}
              label={wbPillLabel(wbStatus)}
            />
          </div>
        {/if}
        <div class="wb-btns">
          <button onclick={enableWriteBlocker} class="btn-sm" disabled={collBusy || busy || !selectedDisk}>Enable Software Write-Blocker</button>
          <button onclick={requestDisableWriteBlocker} class="btn-sm" disabled={collBusy || busy || !selectedDisk || !wbStatus?.software}>Disable</button>
          <button onclick={checkWriteBlocker} class="btn-sm" disabled={collBusy || !selectedDisk}>Refresh</button>
          <button onclick={detectHpaDco} class="btn-sm" disabled={collBusy || busy || hpaBusy || !selectedDisk}>
            {hpaBusy ? "Checking HPA/DCO…" : "Check HPA/DCO"}
          </button>
        </div>
        {#if hpaReport}
          <div class="hpa-report">
            {#if hpaReport.hpaDetected}<PillBadge variant="warning" label="HPA Detected" />{/if}
            {#if hpaReport.dcoDetected}<PillBadge variant="warning" label="DCO Detected" />{/if}
            {#if hpaReport.hiddenSectors != null}<span class="wb-detail">Hidden sectors: {hpaReport.hiddenSectors}</span>{/if}
            <span class="wb-detail">{hpaReport.notes}</span>
          </div>
        {/if}
        {#if wbStatus?.notes}<p class="wb-notes">{wbStatus.notes}</p>{/if}
      </div>
    {/if}
  </MacCard>

  <MacCard title="Format & Hashing">
    <FormatPicker bind:format={imageFormat} bind:hashMd5 bind:hashSha256 disabled={collBusy} />
    <div class="split-row">
      <label>Split (MB): <input type="number" bind:value={splitSize} disabled={collBusy} placeholder="0 = no split" /></label>
      <label class="check"><input type="checkbox" bind:checked={shouldVerify} disabled={collBusy} /> Verify after write</label>
    </div>
  </MacCard>

  <MacCard title="Destination">
    <div class="row">
      <input type="text" bind:value={destPath} disabled={collBusy} class="full" placeholder="/path/to/image.dd" />
      <button onclick={browseDestination} class="btn-sm" disabled={collBusy}>Browse</button>
    </div>
  </MacCard>

  <div class="actions">
    {#if !collBusy}
      <button onclick={requestStartImaging} class="btn-primary" disabled={!selectedDisk}>Start Acquisition</button>
    {:else}
      <button onclick={cancelImaging} class="btn-danger">Stop</button>
    {/if}
  </div>

  {#if imagingSummary}
    <MacCard title="Acquisition Summary">
      <div class="summary-grid">
        <span class="summary-item">Sectors read: {imagingSummary.sectorsRead?.toLocaleString?.() ?? imagingSummary.sectorsRead ?? "—"}</span>
        <span class="summary-item">Duration: {imagingSummary.durationSecs?.toFixed?.(1) ?? imagingSummary.durationSecs ?? "—"}s</span>
        <span class="summary-item">Avg speed: {((imagingSummary.avgSpeedBytesPerSec ?? 0) / 1e6).toFixed(1)} MB/s</span>
        <span class:warn-sector={(imagingSummary.errorSectors ?? 0) > 0}>
          Error sectors: {imagingSummary.errorSectors ?? 0}
          {#if (imagingSummary.errorSectors ?? 0) > 0} (zeroed in image){/if}
        </span>
        <span>Source integrity: {imagingSummary.sourceIntegrityOk ? "OK" : "FAILED"}</span>
        <span class="mono">SHA-256: {imagingSummary.sha256 ?? "—"}</span>
        {#if imagingSummary.badSectorsLog}
          <span class="mono">Bad-sector log: {imagingSummary.badSectorsLog}</span>
        {/if}
      </div>
    </MacCard>
  {/if}

  <GuideCard title={diskImagingGuide.title} icon={diskImagingGuide.icon} steps={diskImagingGuide.steps} references={diskImagingGuide.references} />
</div>

<ConfirmDialog
  open={showConfirmStart}
  title="Start Disk Acquisition?"
  message="This will begin sector-by-sector imaging of the selected drive. Ensure write-blocker is active and the destination has sufficient free space."
  confirmLabel="Start Acquisition"
  variant="primary"
  onConfirm={startImaging}
  onCancel={() => (showConfirmStart = false)}
/>

<ConfirmDialog
  open={showConfirmDisableWb}
  title="Disable Write-Blocker?"
  message="Disabling the software write-blocker allows writes to the source drive. Only proceed if imaging is complete and you intend to modify the device."
  confirmLabel="Disable Write-Blocker"
  variant="danger"
  onConfirm={disableWriteBlocker}
  onCancel={() => (showConfirmDisableWb = false)}
/>

<style>
  .row { display: flex; gap: 8px; align-items: center; }
  .full { flex: 1; }
  select, input {
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
