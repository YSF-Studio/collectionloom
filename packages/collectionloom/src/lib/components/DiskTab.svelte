<script>
import { invoke } from "@tauri-apps/api/core";
import GuideCard from "./GuideCard.svelte";
import { diskImagingGuide } from "../guides.js";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();

let disks = $state([]);
let selectedDisk = $state("");
let destPath = $state("/mnt/evidence/image.dd");
let splitSize = $state("0");
let shouldVerify = $state(true);
let progress = $state({ percent: 0, status: "Idle", bytesProcessed: 0, totalBytes: 0 });
let collBusy = $state(false);
let pollId = $state(null);

// HPA/DCO detection
let hpaDcoResult = $state("");

// ETA tracking
let eta = $state("");
let startTime = $state(null);

// Computed selected disk info
let selectedDiskInfo = $derived(disks.find(d => d.device === selectedDisk) || null);

async function listDisks() {
  setBusy(true);
  try {
    disks = await timeoutPromise(invoke("list_disks"), 15000);
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`❌ ${err}`);
  }
  setBusy(false);
}

async function startImaging() {
  if (!selectedDisk || !destPath) { setMsg("⚠️ Select a disk and destination"); return; }
  collBusy = true;
  startTime = Date.now();
  try {
    await timeoutPromise(invoke("start_disk_imaging", { source: selectedDisk, destination: destPath, splitSizeMb: parseInt(splitSize) || 0, verify: shouldVerify }), 5000);
    // Poll progress
    pollId = setInterval(async () => {
      try {
        const p = await invoke("get_imaging_progress");
        progress = p;
        // Compute ETA
        if (p.bytesProcessed > 0 && startTime) {
          const elapsed = (Date.now() - startTime) / 1000; // seconds
          const speed = p.bytesProcessed / elapsed; // bytes/sec
          if (speed > 0 && p.totalBytes > 0) {
            const remaining = (p.totalBytes - p.bytesProcessed) / speed;
            if (remaining < 60) {
              eta = `${Math.round(remaining)}s`;
            } else if (remaining < 3600) {
              eta = `${Math.round(remaining / 60)}m ${Math.round(remaining % 60)}s`;
            } else {
              eta = `${Math.round(remaining / 3600)}h ${Math.round((remaining % 3600) / 60)}m`;
            }
          }
        }
        if (p.isDone) { clearInterval(pollId); collBusy = false; eta = ""; startTime = null; setMsg(p.error ? `❌ ${p.error}` : "✅ Imaging complete!"); }
      } catch(e) { clearInterval(pollId); collBusy = false; eta = ""; startTime = null; }
    }, 500);
  } catch(e) {
    collBusy = false;
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`❌ ${err}`);
  }
}

async function cancelImaging() {
  await invoke("cancel_imaging");
  if (pollId) clearInterval(pollId);
  collBusy = false;
  progress.status = "Cancelled";
  eta = "";
  startTime = null;
}

async function detectHpaDco() {
  if (!selectedDisk) { setMsg("⚠️ Select a disk first"); return; }
  hpaDcoResult = "Detecting...";
  try {
    const result = await timeoutPromise(invoke("hpa_dco_detect", { device: selectedDisk }), 30000);
    hpaDcoResult = result;
  } catch(e) {
    hpaDcoResult = `❌ ${typeof e === "string" ? e : String(e)}`;
  }
}

// Load on mount
$effect(() => { listDisks(); });
</script>

<div>
  <h3>💿 Disk Acquisition</h3>
  
  <div class="row">
    <label>Source Device:
      <select bind:value={selectedDisk} disabled={collBusy||busy}>
        <option value="">-- Select disk --</option>
        {#each disks as disk}
          <option value={disk.device}>{disk.device} ({disk.model}) — {(disk.sizeBytes/1e9).toFixed(1)} GB {disk.isSsd ? "SSD" : "HDD"}</option>
        {/each}
      </select>
    </label>
    <button onclick={listDisks} class="btn-sm">🔄 Refresh</button>
  </div>

  {#if selectedDiskInfo}
  <div class="device-detail">
    <span class="detail-item">Model: {selectedDiskInfo.model || "Unknown"}</span>
    <span class="detail-item">Size: {(selectedDiskInfo.sizeBytes / 1e9).toFixed(1)} GB</span>
    <span class="detail-item">Interface: {selectedDiskInfo.interfaceType || "Unknown"}</span>
  </div>
  {/if}

  {#if selectedDiskInfo && selectedDiskInfo.isSsd}
  <div class="ssd-warning">
    ⚠️ SSD TRIM may have erased deleted data. Consider hardware write blocker.
  </div>
  {/if}

  <div class="row">
    <label>Destination: <input type="text" bind:value={destPath} disabled={collBusy} placeholder="/mnt/evidence/image.dd" /></label>
  </div>

  <div class="options">
    <label>Split (MB): <input type="number" bind:value={splitSize} disabled={collBusy} placeholder="0=no split" style="width:80px" /></label>
    <label><input type="checkbox" bind:checked={shouldVerify} disabled={collBusy} /> Verify after write</label>
  </div>

  <div class="actions">
    {#if !collBusy}
      <button onclick={startImaging} class="btn-primary" disabled={!selectedDisk}>▶️ Start Collection</button>
    {:else}
      <button onclick={cancelImaging} class="btn-danger">■ Stop</button>
    {/if}
    <button onclick={detectHpaDco} class="btn-sm" disabled={!selectedDisk || collBusy}>🔍 Detect HPA/DCO</button>
  </div>

  {#if hpaDcoResult}
  <div class="hpa-result">
    <pre>{hpaDcoResult}</pre>
  </div>
  {/if}

  {#if collBusy || progress.bytesProcessed > 0}
  <div class="progress-bar">
    <div class="fill" style="width:{progress.percent}%"></div>
  </div>
  <div class="progress-info">
    <span>{progress.percent.toFixed(1)}%</span>
    <span>{progress.status}</span>
    <span>{(progress.bytesProcessed / 1e9).toFixed(2)} GB</span>
    {#if eta}<span>ETA: {eta}</span>{/if}
  </div>
  {/if}

  <GuideCard title={diskImagingGuide.title} icon={diskImagingGuide.icon} steps={diskImagingGuide.steps} references={diskImagingGuide.references} />
</div>

<style>
h3 { margin: 0 0 16px; font-size: 16px; }
.row { display: flex; gap: 10px; align-items: center; margin-bottom: 12px; }
label { font-size: 13px; display: flex; align-items: center; gap: 6px; }
select, input { background: #1a1a1a; color: #e0e0e0; border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; font-size: 13px; width: 100%; }
.options { display: flex; gap: 16px; margin: 12px 0; font-size: 13px; }
.actions { margin: 16px 0; display: flex; gap: 8px; align-items: center; }
.btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 600; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger { padding: 10px 24px; background: var(--danger); color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; }
.btn-sm { padding: 5px 10px; background: var(--border); color: #e0e0e0; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; }
.btn-sm:disabled { opacity: 0.5; cursor: not-allowed; }
.progress-bar { height: 8px; background: #2a2a2a; border-radius: 4px; margin: 12px 0; overflow: hidden; }
.fill { height: 100%; background: var(--primary); border-radius: 4px; transition: width 0.3s; }
.progress-info { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-secondary); gap: 12px; }
.ssd-warning { background: rgba(245,158,11,0.15); border: 1px solid rgba(245,158,11,0.3); border-radius: 8px; padding: 10px 14px; font-size: 12px; color: var(--warn, #f59e0b); margin-bottom: 12px; }
.device-detail { display: flex; gap: 16px; margin-bottom: 12px; font-size: 12px; color: var(--text-secondary); }
.detail-item { background: #1a1a1a; border: 1px solid var(--border); border-radius: 6px; padding: 4px 10px; }
.hpa-result { background: #1a1a1a; border: 1px solid var(--border); border-radius: 8px; padding: 10px; margin: 12px 0; }
.hpa-result pre { margin: 0; font-size: 12px; color: var(--text-secondary); white-space: pre-wrap; word-break: break-all; }
</style>
