<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import { acquireAllGuide } from "../guides.js";

let { sharedState, caseState = {}, busy, setBusy, setMsg, timeoutPromise, onProgressChange = () => {}, onDeviceSelect = () => {} } = $props();

let diskEnabled = $state(false);
let ramEnabled = $state(false);
let networkEnabled = $state(false);
let mobileEnabled = $state(false);
let cloudEnabled = $state(false);

let devices = $state([]);
let selectedDevice = $state("");
let splitSizeMb = $state("4096");
let wbStatus = $state(null);
let ramTools = $state([]);
let selectedRamTool = $state("");
let interfaces = $state([]);
let selectedIface = $state("");
let bpfFilter = $state("");
let outputFolder = $state("/tmp/forensic_case/");
let mobileDetected = $state(false);
let mobileDeviceId = $state("");
let cloudConfigured = $state(false);

let running = $state(false);
let moduleProgress = $state({
  disk: { percent: 0, status: "Idle", eta: "" },
  ram: { percent: 0, status: "Idle", eta: "" },
  network: { percent: 0, status: "Idle", eta: "" },
  mobile: { percent: 0, status: "Idle", eta: "" },
  cloud: { percent: 0, status: "Idle", eta: "" },
});
let moduleDetails = $state({ disk: "", ram: "", network: "", mobile: "", cloud: "" });

const moduleOrder = [
  { key: "network", label: "Network", enabled: () => networkEnabled },
  { key: "ram", label: "RAM", enabled: () => ramEnabled },
  { key: "mobile", label: "Mobile", enabled: () => mobileEnabled },
  { key: "disk", label: "Disk", enabled: () => diskEnabled },
  { key: "cloud", label: "Cloud", enabled: () => cloudEnabled },
];

let selectedDiskInfo = $derived(devices.find((d) => d.device === selectedDevice) || null);

function formatSize(bytes) {
  if (!bytes) return "unknown size";
  if (bytes >= 1e12) return `${(bytes / 1e12).toFixed(2)} TB`;
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
  return `${(bytes / 1e6).toFixed(0)} MB`;
}

function effectiveSplitMb() {
  const n = parseInt(splitSizeMb, 10);
  if (n > 0) return n;
  const sz = selectedDiskInfo?.sizeBytes || 0;
  if (sz > 4_294_967_296) return 4096;
  return 0;
}

async function refreshWriteBlocker() {
  if (!selectedDevice) {
    wbStatus = null;
    onDeviceSelect({ device: "", wbActive: false });
    return;
  }
  try {
    wbStatus = await timeoutPromise(invoke("check_write_blocker", { device: selectedDevice }), 5000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDevice, wbActive: active });
  } catch {
    wbStatus = null;
    onDeviceSelect({ device: selectedDevice, wbActive: false });
  }
}

async function enableWriteBlocker() {
  if (!selectedDevice) {
    setMsg("WARN: Select a disk first");
    return;
  }
  setBusy(true);
  try {
    wbStatus = await timeoutPromise(invoke("enable_write_blocker", { device: selectedDevice }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDevice, wbActive: active });
    setMsg(active ? "OK: Software write-blocker enabled" : "WARN: Write-blocker not confirmed active");
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function detectModules() {
  setBusy(true);
  try {
    devices = await timeoutPromise(invoke("list_disks"), 5000).catch(() => []);
    interfaces = await timeoutPromise(invoke("list_interfaces"), 5000).catch(() => []);
    ramTools = await timeoutPromise(invoke("list_ram_tools"), 5000).catch(() => []);
    if (ramTools.length) selectedRamTool = ramTools[0];

    const android = await timeoutPromise(invoke("list_android_devices"), 5000).catch(() => []);
    const ios = await timeoutPromise(invoke("list_ios_devices"), 5000).catch(() => []);
    const allMobile = [...(android || []), ...(ios || [])];
    mobileDetected = allMobile.length > 0;
    if (mobileDetected) mobileDeviceId = allMobile[0].id || allMobile[0].device_id || "";

    cloudConfigured = false;
    if (selectedDevice) await refreshWriteBlocker();
  } catch {
    /* best effort */
  }
  setBusy(false);
}

async function pollDiskProgress() {
  return new Promise((resolve) => {
    const id = setInterval(async () => {
      try {
        const p = await invoke("get_imaging_progress");
        moduleProgress.disk = {
          percent: p.percent || 0,
          status: "Running",
          eta: "",
        };
        moduleDetails.disk = `${(p.percent || 0).toFixed(1)}% — ${p.status}`;
        onProgressChange({ label: `Imaging ${selectedDevice}`, percent: p.percent, busy: true });
        if (p.isDone) {
          clearInterval(id);
          moduleProgress.disk = { percent: 100, status: p.error ? "Failed" : "Done", eta: "" };
          moduleDetails.disk = p.error || "Complete";
          resolve(p.error);
        }
      } catch {
        clearInterval(id);
        resolve("Progress poll failed");
      }
    }, 500);
  });
}

async function ensureWriteBlocker() {
  await refreshWriteBlocker();
  if (wbStatus?.active || wbStatus?.enabled) return;
  try {
    wbStatus = await timeoutPromise(invoke("enable_write_blocker", { device: selectedDevice }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDevice, wbActive: active });
    if (!active) {
      throw new Error("Write-blocker could not be activated — enable manually or use hardware blocker");
    }
    moduleDetails.disk = "Software write-blocker enabled";
  } catch (e) {
    throw new Error(typeof e === "string" ? e : String(e));
  }
}

async function checkStorage(outputPath, sourceDevice = null) {
  const report = await timeoutPromise(
    invoke("verify_acquisition_storage", {
      output: outputPath,
      sourceDevice,
    }),
    10000
  );
  if (!report.ok) {
    throw new Error(`Storage check: ${report.notes}`);
  }
  return report;
}

async function recordEvidenceHash(path, moduleKey) {
  try {
    const report = await timeoutPromise(invoke("hash_and_verify_evidence", { path }), 120000);
    const verified = report.verified ? "verified" : "NOT verified";
    moduleDetails[moduleKey] = `SHA-256: ${report.sha256.slice(0, 16)}… (${verified})`;
    return report;
  } catch (e) {
    const msg = typeof e === "string" ? e : String(e);
    moduleDetails[moduleKey] = `${moduleDetails[moduleKey] || path} · hash failed: ${msg}`;
    return null;
  }
}

async function startAcquireAll() {
  running = true;
  setBusy(true);
  for (const mod of moduleOrder) {
    if (mod.enabled()) {
      moduleProgress[mod.key] = { percent: 0, status: "Pending", eta: "" };
      moduleDetails[mod.key] = "";
    }
  }

  for (const mod of moduleOrder.filter((m) => m.enabled())) {
    const key = mod.key;
    moduleProgress[key].status = "Running";
    try {
      if (key === "disk") await runDiskAcquisition();
      else if (key === "ram") await runRamAcquisition();
      else if (key === "network") await runNetworkAcquisition();
      else if (key === "mobile") await runMobileAcquisition();
      else if (key === "cloud") {
        moduleProgress.cloud = { percent: 0, status: "Skipped", eta: "" };
        moduleDetails.cloud = "Configure cloud credentials in Cloud Snapshot tab";
      }
    } catch (e) {
      moduleProgress[key].status = "Failed";
      moduleDetails[key] = typeof e === "string" ? e : String(e);
    }
  }

  running = false;
  setBusy(false);
  onProgressChange({ busy: false });
  setMsg("OK: Batch acquisition complete");
}

async function runDiskAcquisition() {
  if (!selectedDevice) {
    moduleProgress.disk = { percent: 0, status: "Failed", eta: "" };
    moduleDetails.disk = "No device selected";
    return;
  }
  await ensureWriteBlocker();
  const split = effectiveSplitMb();
  const dest = outputFolder.replace(/\/$/, "") + "/disk_image.dd";
  moduleDetails.disk = `Split: ${split > 0 ? split + " MB" : "none"} · ${formatSize(selectedDiskInfo?.sizeBytes)}`;
  await invoke("start_disk_imaging", {
    source: selectedDevice,
    destination: dest,
    splitSizeMb: split,
    verify: true,
    imageFormat: "raw",
    caseId: caseState.caseId || null,
    operator: caseState.operator || null,
  });
  const err = await pollDiskProgress();
  if (err) throw new Error(String(err));
}

async function runRamAcquisition() {
  const dest = outputFolder.replace(/\/$/, "") + "/ram_dump.lime";
  await checkStorage(dest);
  moduleProgress.ram = { percent: 50, status: "Running", eta: "" };
  const result = await timeoutPromise(
    invoke("capture_ram", {
      tool: selectedRamTool || "Avml",
      output: dest,
      compress: true,
      caseId: caseState.caseId || null,
      operator: caseState.operator || null,
    }),
    120000
  );
  moduleProgress.ram = { percent: 100, status: "Done", eta: "" };
  if (result?.sha256) {
    const verified = result.verified ? "verified" : "NOT verified";
    moduleDetails.ram = `SHA-256: ${result.sha256.slice(0, 16)}… (${verified})`;
  } else {
    await recordEvidenceHash(dest, "ram");
  }
}

async function runNetworkAcquisition() {
  const dest = outputFolder.replace(/\/$/, "") + "/network.pcapng";
  await checkStorage(dest);
  moduleProgress.network = { percent: 30, status: "Running", eta: "" };
  await timeoutPromise(
    invoke("start_network_capture", {
      interface: selectedIface,
      bpfFilter: bpfFilter || null,
      outputFile: dest,
    }),
    5000
  );
  await sleep(5000);
  await invoke("cancel_network_capture");
  moduleProgress.network = { percent: 100, status: "Done", eta: "" };
  await recordEvidenceHash(dest, "network");
}

async function runMobileAcquisition() {
  if (!mobileDetected || !mobileDeviceId) {
    moduleProgress.mobile = { percent: 0, status: "Skipped", eta: "" };
    moduleDetails.mobile = "No device detected";
    return;
  }
  const dest = outputFolder.replace(/\/$/, "") + "/mobile_backup.ab";
  await checkStorage(dest);
  moduleProgress.mobile = { percent: 50, status: "Running", eta: "" };
  try {
    await timeoutPromise(
      invoke("adb_backup", { deviceId: mobileDeviceId, output: dest }),
      120000
    );
    moduleProgress.mobile = { percent: 100, status: "Done", eta: "" };
    await recordEvidenceHash(dest, "mobile");
  } catch (e) {
    moduleProgress.mobile = { percent: 0, status: "Failed", eta: "" };
    throw e;
  }
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

$effect(() => {
  detectModules();
});

$effect(() => {
  if (selectedDevice) refreshWriteBlocker();
});
</script>

<div class="acquire-all-tab">
  <SectionHeader title="Acquire All" subtitle="Run selected acquisition modules in sequence" />

  <div class="modules-grid">
    <MacCard title="Disk">
      <label class="toggle"><input type="checkbox" bind:checked={diskEnabled} /> Enable</label>
      {#if diskEnabled}
        <select bind:value={selectedDevice} class="full">
          <option value="">— Select device —</option>
          {#each devices as d}
            <option value={d.device}>{d.device} — {formatSize(d.sizeBytes)} {d.model || ""}</option>
          {/each}
        </select>
        <label class="split-label">Split (MB, 0 = auto for drives &gt;4 GB):</label>
        <input type="number" bind:value={splitSizeMb} class="full" placeholder="4096" />
        {#if wbStatus}
          <div class="wb-row">
            <PillBadge variant={wbStatus.active ? "active" : "inactive"} label={wbStatus.active ? "Protected" : "Not protected"} />
            <span class="wb-note">{wbStatus.method}{wbStatus.hardware ? " (hardware)" : ""}{wbStatus.software ? " (software)" : ""}</span>
          </div>
        {/if}
        <div class="wb-actions">
          <button class="btn-sm" onclick={enableWriteBlocker} disabled={busy || !selectedDevice}>Enable Software Write-Blocker</button>
          <button class="btn-sm" onclick={refreshWriteBlocker} disabled={busy || !selectedDevice}>Refresh</button>
        </div>
      {/if}
    </MacCard>

    <MacCard title="RAM">
      <label class="toggle"><input type="checkbox" bind:checked={ramEnabled} /> Enable</label>
      {#if ramEnabled}
        <select bind:value={selectedRamTool} class="full">
          {#each ramTools as t}<option value={t}>{t}</option>{/each}
        </select>
      {/if}
    </MacCard>

    <MacCard title="Network">
      <label class="toggle"><input type="checkbox" bind:checked={networkEnabled} /> Enable</label>
      {#if networkEnabled}
        <select bind:value={selectedIface} class="full">
          <option value="">— Interface —</option>
          {#each interfaces as i}<option value={i}>{i}</option>{/each}
        </select>
        <input bind:value={bpfFilter} placeholder="BPF filter" class="full" />
      {/if}
    </MacCard>

    <MacCard title="Mobile">
      <label class="toggle"><input type="checkbox" bind:checked={mobileEnabled} disabled={!mobileDetected} /> Enable</label>
      <span class="hint">{mobileDetected ? "Device detected" : "No device detected"}</span>
    </MacCard>
  </div>

  <MacCard title="Output">
    <div class="row">
      <input bind:value={outputFolder} class="full" />
      <button class="btn-sm" onclick={detectModules}>Detect Devices</button>
    </div>
    <p class="hint">Use NTFS/APFS/ext4 for multi-TB images; enable split on FAT32 destinations.</p>
  </MacCard>

  <button class="btn-acquire" onclick={startAcquireAll} disabled={running || busy || !moduleOrder.some((m) => m.enabled())}>
    {running ? "Acquiring…" : "Start Acquire All"}
  </button>

  {#if moduleOrder.some((m) => moduleProgress[m.key].status !== "Idle")}
    <MacCard title="Progress">
      {#each moduleOrder as mod}
        {@const prog = moduleProgress[mod.key]}
        {#if prog.status !== "Idle"}
          <div class="prog-row">
            <span class="prog-label">{mod.label}</span>
            <span class="prog-status">{prog.status}</span>
            {#if prog.percent > 0 && prog.status === "Running"}
              <div class="bar"><div class="fill" style="width:{prog.percent}%"></div></div>
            {/if}
            {#if moduleDetails[mod.key]}<span class="detail">{moduleDetails[mod.key]}</span>{/if}
          </div>
        {/if}
      {/each}
    </MacCard>
  {/if}

  <GuideCard title={acquireAllGuide.title} icon={acquireAllGuide.icon} steps={acquireAllGuide.steps} references={acquireAllGuide.references} />
</div>

<style>
  .acquire-all-tab { max-width: 780px; }
  .modules-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px; }
  .toggle { display: flex; gap: 8px; font-size: 13px; margin-bottom: 8px; cursor: pointer; }
  .split-label { font-size: 11px; color: var(--text-muted); display: block; margin-bottom: 4px; }
  .full { width: 100%; background: var(--input-bg); border: 1px solid var(--border); border-radius: 8px; padding: 6px 10px; color: var(--text); font-size: 12px; margin-bottom: 6px; }
  .hint { font-size: 11px; color: var(--text-muted); margin: 4px 0 0; }
  .wb-row { display: flex; align-items: center; gap: 8px; margin: 6px 0; flex-wrap: wrap; }
  .wb-note { font-size: 11px; color: var(--text-secondary); }
  .wb-actions { display: flex; gap: 8px; margin-bottom: 4px; }
  .row { display: flex; gap: 8px; }
  .btn-sm { padding: 6px 12px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 6px; color: var(--btn-secondary-text); cursor: pointer; white-space: nowrap; font-size: 12px; }
  .btn-acquire { width: 100%; padding: 12px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 700; cursor: pointer; margin-bottom: 16px; }
  .btn-acquire:disabled { opacity: 0.4; }
  .prog-row { margin-bottom: 10px; font-size: 12px; }
  .prog-label { font-weight: 600; margin-right: 8px; }
  .prog-status { color: var(--text-secondary); }
  .bar { height: 6px; background: var(--btn-secondary-bg); border-radius: 3px; margin: 4px 0; overflow: hidden; }
  .fill { height: 100%; background: var(--primary); transition: width 0.3s; }
  .detail { display: block; font-size: 11px; color: var(--text-muted); margin-top: 2px; }
</style>
