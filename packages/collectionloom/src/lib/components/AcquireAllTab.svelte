<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import { snapshotGuide } from "../guides.js";

let { sharedState, busy, setBusy, setMsg, timeoutPromise, onProgressChange = () => {} } = $props();

let diskEnabled = $state(false);
let ramEnabled = $state(false);
let networkEnabled = $state(false);
let mobileEnabled = $state(false);
let cloudEnabled = $state(false);

let devices = $state([]);
let selectedDevice = $state("");
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
  { key: "disk", label: "Disk", enabled: () => diskEnabled },
  { key: "ram", label: "RAM", enabled: () => ramEnabled },
  { key: "network", label: "Network", enabled: () => networkEnabled },
  { key: "mobile", label: "Mobile", enabled: () => mobileEnabled },
  { key: "cloud", label: "Cloud", enabled: () => cloudEnabled },
];

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
  } catch {
    /* best effort */
  }
  setBusy(false);
}

async function pollDiskProgress(startTime) {
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
  setMsg("Batch acquisition complete");
}

async function runDiskAcquisition() {
  if (!selectedDevice) {
    moduleProgress.disk = { percent: 0, status: "Failed", eta: "" };
    moduleDetails.disk = "No device selected";
    return;
  }
  const dest = outputFolder.replace(/\/$/, "") + "/disk_image.dd";
  await invoke("start_disk_imaging", {
    source: selectedDevice,
    destination: dest,
    splitSizeMb: 0,
    verify: true,
    imageFormat: "raw",
  });
  const err = await pollDiskProgress(Date.now());
  if (err) throw new Error(String(err));
}

async function runRamAcquisition() {
  const dest = outputFolder.replace(/\/$/, "") + "/ram_dump.lime";
  moduleProgress.ram = { percent: 50, status: "Running", eta: "" };
  const hash = await timeoutPromise(
    invoke("capture_ram", { tool: selectedRamTool || "Avml", output: dest, compress: true }),
    120000
  );
  moduleProgress.ram = { percent: 100, status: "Done", eta: "" };
  moduleDetails.ram = hash ? `Saved · ${hash.slice(0, 16)}…` : "Complete";
}

async function runNetworkAcquisition() {
  const dest = outputFolder.replace(/\/$/, "") + "/network.pcapng";
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
  moduleDetails.network = dest;
}

async function runMobileAcquisition() {
  if (!mobileDetected || !mobileDeviceId) {
    moduleProgress.mobile = { percent: 0, status: "Skipped", eta: "" };
    moduleDetails.mobile = "No device detected";
    return;
  }
  const dest = outputFolder.replace(/\/$/, "") + "/mobile_backup.ab";
  moduleProgress.mobile = { percent: 50, status: "Running", eta: "" };
  try {
    await timeoutPromise(
      invoke("adb_backup", { deviceId: mobileDeviceId, output: dest }),
      120000
    );
    moduleProgress.mobile = { percent: 100, status: "Done", eta: "" };
    moduleDetails.mobile = dest;
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
            <option value={d.device}>{d.device} — {(d.sizeBytes / 1e9).toFixed(1)} GB</option>
          {/each}
        </select>
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

  <GuideCard title={snapshotGuide.title} icon={snapshotGuide.icon} steps={snapshotGuide.steps} references={snapshotGuide.references} />
</div>

<style>
  .acquire-all-tab { max-width: 780px; }
  .modules-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px; }
  .toggle { display: flex; gap: 8px; font-size: 13px; margin-bottom: 8px; cursor: pointer; }
  .full { width: 100%; background: var(--input-bg); border: 1px solid var(--border); border-radius: 8px; padding: 6px 10px; color: var(--text); font-size: 12px; margin-bottom: 6px; }
  .hint { font-size: 11px; color: var(--text-muted); }
  .row { display: flex; gap: 8px; }
  .btn-sm { padding: 6px 12px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 6px; color: var(--btn-secondary-text); cursor: pointer; white-space: nowrap; }
  .btn-acquire { width: 100%; padding: 12px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 700; cursor: pointer; margin-bottom: 16px; }
  .btn-acquire:disabled { opacity: 0.4; }
  .prog-row { margin-bottom: 10px; font-size: 12px; }
  .prog-label { font-weight: 600; margin-right: 8px; }
  .prog-status { color: var(--text-secondary); }
  .bar { height: 6px; background: var(--btn-secondary-bg); border-radius: 3px; margin: 4px 0; overflow: hidden; }
  .fill { height: 100%; background: var(--primary); transition: width 0.3s; }
  .detail { display: block; font-size: 11px; color: var(--text-muted); margin-top: 2px; }
</style>
