<script>
import { invoke } from "../api/tauri.js";
import { getPortableLayout, joinPortablePath } from "../api/portable.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import PillBadge from "./ui/PillBadge.svelte";
import ConfirmDialog from "./ui/ConfirmDialog.svelte";
import { acquireAllGuide } from "../guides.js";
import { wbPillLabel } from "../wb.js";
import { getResolvedLocale, subscribeLocale } from "../stores/locale.js";

let { sharedState, caseState = {}, busy, setBusy, setMsg, timeoutPromise, onProgressChange = () => {}, onDeviceSelect = () => {} } = $props();
let locale = $state(getResolvedLocale());

$effect(() => subscribeLocale((_, resolved) => {
  locale = resolved;
}));

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
let outputFolder = $state("");
let kitRoot = $state("");
let mobileDetected = $state(false);
let mobileDeviceId = $state("");
let cloudConfigured = $state(false);

let running = $state(false);
let detecting = $state(false);
let showConfirmAcquire = $state(false);
let moduleProgress = $state({
  disk: { percent: 0, status: "Idle", eta: "" },
  ram: { percent: 0, status: "Idle", eta: "" },
  network: { percent: 0, status: "Idle", eta: "" },
  mobile: { percent: 0, status: "Idle", eta: "" },
  cloud: { percent: 0, status: "Idle", eta: "" },
});
let moduleDetails = $state({ disk: "", ram: "", network: "", mobile: "", cloud: "" });

const moduleOrder = [
  { key: "network", label: () => (locale === "id" ? "Jaringan" : "Network"), enabled: () => networkEnabled },
  { key: "ram", label: () => "RAM", enabled: () => ramEnabled },
  { key: "mobile", label: () => "Mobile", enabled: () => mobileEnabled },
  { key: "disk", label: () => "Disk", enabled: () => diskEnabled },
  { key: "cloud", label: () => "Cloud", enabled: () => cloudEnabled },
];

let selectedDiskInfo = $derived(devices.find((d) => d.device === selectedDevice) || null);

function formatSize(bytes) {
  if (!bytes) return locale === "id" ? "ukuran tidak diketahui" : "unknown size";
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
    setMsg(locale === "id" ? "PERINGATAN: Pilih disk terlebih dahulu" : "WARN: Select a disk first");
    return;
  }
  setBusy(true);
  try {
    wbStatus = await timeoutPromise(invoke("enable_write_blocker", { device: selectedDevice }), 15000);
    const active = wbStatus?.active ?? wbStatus?.enabled ?? false;
    onDeviceSelect({ device: selectedDevice, wbActive: active });
    setMsg(active ? (locale === "id" ? "OK: Software write-blocker aktif" : "OK: Software write-blocker enabled") : (locale === "id" ? "PERINGATAN: Write-blocker belum aktif" : "WARN: Write-blocker not confirmed active"));
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function detectModules() {
  detecting = true;
  setBusy(true);
  try {
    devices = await timeoutPromise(invoke("list_disks"), 5000).catch(() => []);
    interfaces = await timeoutPromise(invoke("list_interfaces"), 5000).catch(() => []);
    ramTools = await timeoutPromise(invoke("list_ram_tools"), 5000).catch(() => []);
    if (ramTools.length) selectedRamTool = ramTools[0];

    const android = await timeoutPromise(invoke("list_android_devices"), 5000).catch(() => []);
    const ios = await timeoutPromise(invoke("list_ios_devices"), 5000).catch(() => []);
    const allMobile = [...(android || []), ...(ios || [])];
    const wasMobileDetected = mobileDetected;
    mobileDetected = allMobile.length > 0;
    if (mobileDetected) {
      mobileDeviceId = allMobile[0].id || allMobile[0].device_id || "";
      if (!wasMobileDetected) mobileEnabled = true;
    } else {
      mobileEnabled = false;
    }

    cloudConfigured = false;
    if (selectedDevice) await refreshWriteBlocker();
  } catch {
    /* best effort */
  }
  detecting = false;
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
      throw new Error(locale === "id" ? "Write-blocker tidak dapat diaktifkan — aktifkan manual atau gunakan hardware blocker" : "Write-blocker could not be activated — enable manually or use hardware blocker");
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
    throw new Error(locale === "id" ? `Pemeriksaan storage: ${report.notes}` : `Storage check: ${report.notes}`);
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

function requestStartAcquireAll() {
  showConfirmAcquire = true;
}

async function startAcquireAll() {
  showConfirmAcquire = false;
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
        moduleDetails.cloud = locale === "id" ? "Atur kredensial cloud di tab Cloud Snapshot" : "Configure cloud credentials in Cloud Snapshot tab";
      }
    } catch (e) {
      moduleProgress[key].status = "Failed";
      moduleDetails[key] = typeof e === "string" ? e : String(e);
    }
  }

  running = false;
  setBusy(false);
  onProgressChange({ busy: false });
  setMsg(locale === "id" ? "OK: Akuisisi batch selesai" : "OK: Batch acquisition complete");
}

async function runDiskAcquisition() {
  if (!selectedDevice) {
    moduleProgress.disk = { percent: 0, status: "Failed", eta: "" };
    moduleDetails.disk = locale === "id" ? "Tidak ada perangkat dipilih" : "No device selected";
    return;
  }
  await ensureWriteBlocker();
  const split = effectiveSplitMb();
  const dest = joinPortablePath(outputFolder, "disk_image.dd");
  moduleDetails.disk = `${locale === "id" ? "Pecah" : "Split"}: ${split > 0 ? split + " MB" : (locale === "id" ? "tidak ada" : "none")} · ${formatSize(selectedDiskInfo?.sizeBytes)}`;
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
  const dest = joinPortablePath(outputFolder, "ram_dump.lime");
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
  const dest = joinPortablePath(outputFolder, "network.pcapng");
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
  const dest = joinPortablePath(outputFolder, "mobile_backup.ab");
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
  getPortableLayout().then((layout) => {
    kitRoot = layout.kitRoot || layout.defaultAcquisitionDir || "";
    if (!outputFolder) {
      outputFolder = layout.defaultAcquisitionDir + layout.pathSeparator;
    }
  });
  detectModules();
});

$effect(() => {
  if (selectedDevice) refreshWriteBlocker();
});
</script>

<div class="tab-content acquire-all-tab">
  <SectionHeader title={locale === "id" ? "Akuisisi Semua" : "Acquire All"} subtitle={locale === "id" ? "Jalankan modul akuisisi yang dipilih secara berurutan" : "Run selected acquisition modules in sequence"} />

  <div class="sources-toolbar">
    <button class="btn-sm" onclick={detectModules} disabled={detecting || running}>
      {#if detecting}<span class="spinner">↻</span>{/if}
      {detecting ? (locale === "id" ? "Mendeteksi…" : "Detecting…") : (locale === "id" ? "Deteksi Sumber" : "Detect Sources")}
    </button>
    <p class="hint">{locale === "id" ? "Menyegarkan daftar sumber untuk modul disk, RAM, jaringan, dan mobile — bukan folder output atau file kit." : "Refreshes source lists for disk, RAM, network, and mobile modules — not the output folder or kit files."}</p>
  </div>

  <MacCard title={locale === "id" ? "Tujuan" : "Destination"}>
    <div class="output-row">
      <label for="acquire-all-output">{locale === "id" ? "Folder keluaran bukti" : "Evidence output folder"}</label>
      <input
        id="acquire-all-output"
        bind:value={outputFolder}
        class="full"
        placeholder={locale === "id" ? "Pilih folder akuisisi portable" : "Choose a portable acquisition folder"}
      />
    </div>
    <p class="hint">{locale === "id" ? "Semua output modul ditulis ke sini. Ini harus berada di penyimpanan penyidik, bukan di perangkat sumber atau flashdisk kit." : "All module outputs are written here. This should be on the investigator's storage, not the source device or the kit flashdisk."}</p>
  </MacCard>

  <div class="modules-grid">
    <MacCard title={locale === "id" ? "Disk" : "Disk"} class="module-card">
      <label class="toggle"><input type="checkbox" bind:checked={diskEnabled} /> {locale === "id" ? "Aktifkan" : "Enable"}</label>
      {#if diskEnabled}
        <select bind:value={selectedDevice} class="full">
          <option value="">{locale === "id" ? "— Pilih perangkat —" : "— Select device —"}</option>
          {#each devices as d}
            <option value={d.device}>{d.device} — {formatSize(d.sizeBytes)} {d.model || ""}</option>
          {/each}
        </select>
        <label class="split-label" for="acquire-all-split">{locale === "id" ? "Pecah (MB, 0 = otomatis untuk drive &gt;4 GB):" : "Split (MB, 0 = auto for drives &gt;4 GB):"}</label>
        <input id="acquire-all-split" type="number" bind:value={splitSizeMb} class="full" placeholder="4096" />
        {#if wbStatus}
          <div class="wb-row">
            <span class="wb-label">{locale === "id" ? "Write-Blocker:" : "Write-Blocker:"}</span>
            <PillBadge
              variant={(wbStatus.active ?? wbStatus.enabled) ? "active" : "inactive"}
              label={wbPillLabel(wbStatus)}
            />
          </div>
        {/if}
        <div class="wb-actions">
          <button class="btn-sm" onclick={enableWriteBlocker} disabled={busy || !selectedDevice}>{locale === "id" ? "Aktifkan Software Write-Blocker" : "Enable Software Write-Blocker"}</button>
          <button class="btn-sm" onclick={refreshWriteBlocker} disabled={busy || !selectedDevice}>{locale === "id" ? "Segarkan" : "Refresh"}</button>
        </div>
      {/if}
    </MacCard>

    <MacCard title="RAM" class="module-card">
      <label class="toggle"><input type="checkbox" bind:checked={ramEnabled} /> {locale === "id" ? "Aktifkan" : "Enable"}</label>
      {#if ramEnabled}
        <select bind:value={selectedRamTool} class="full">
          {#each ramTools as t}<option value={t}>{t}</option>{/each}
        </select>
      {/if}
    </MacCard>

    <MacCard title={locale === "id" ? "Jaringan" : "Network"} class="module-card">
      <label class="toggle"><input type="checkbox" bind:checked={networkEnabled} /> {locale === "id" ? "Aktifkan" : "Enable"}</label>
      {#if networkEnabled}
        <select bind:value={selectedIface} class="full">
          <option value="">{locale === "id" ? "— Antarmuka —" : "— Interface —"}</option>
          {#each interfaces as i}<option value={i}>{i}</option>{/each}
        </select>
        <input bind:value={bpfFilter} placeholder={locale === "id" ? "Filter BPF" : "BPF filter"} class="full" />
      {/if}
    </MacCard>

    <MacCard title="Mobile" class="module-card">
      <label class="toggle"><input type="checkbox" bind:checked={mobileEnabled} disabled={!mobileDetected} /> {locale === "id" ? "Aktifkan" : "Enable"}</label>
      {#if mobileEnabled}
        <span class="hint">{mobileDetected ? (locale === "id" ? "Perangkat terdeteksi" : "Device detected") : (locale === "id" ? "Tidak ada perangkat terdeteksi" : "No device detected")}</span>
      {/if}
    </MacCard>
  </div>

  <MacCard title={locale === "id" ? "Kit Portable" : "Portable Kit"}>
    <input value={kitRoot} class="full" readonly />
    <p class="hint">{locale === "id" ? "Ini adalah lokasi kit USB/portable CollectionLoom. Terpisah dari folder tujuan bukti." : "This is the CollectionLoom USB/portable kit location. It is separate from the evidence destination folder."}</p>
  </MacCard>

  <button class="btn-acquire" onclick={requestStartAcquireAll} disabled={running || busy || !moduleOrder.some((m) => m.enabled())}>
    {running ? (locale === "id" ? "Mengakuisisi…" : "Acquiring…") : (locale === "id" ? "Mulai Akuisisi Semua" : "Start Acquire All")}
  </button>

  {#if moduleOrder.some((m) => moduleProgress[m.key].status !== "Idle")}
    <MacCard title={locale === "id" ? "Progres" : "Progress"}>
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

<ConfirmDialog
  open={showConfirmAcquire}
  title={locale === "id" ? "Mulai Akuisisi Semua?" : "Start Acquire All?"}
  message={locale === "id" ? "Ini akan menjalankan semua modul akuisisi yang aktif secara berurutan. Pastikan perangkat sumber terlindungi dan tujuan bukti memiliki ruang yang cukup." : "This will run all enabled acquisition modules in sequence. Ensure the source device is protected and the evidence destination has sufficient space."}
  confirmLabel={locale === "id" ? "Mulai Akuisisi" : "Start Acquisition"}
  variant="primary"
  onConfirm={startAcquireAll}
  onCancel={() => (showConfirmAcquire = false)}
/>

<style>
  .sources-toolbar { margin-bottom: 12px; }
  .modules-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-auto-rows: 1fr;
    gap: 12px;
    margin-bottom: 12px;
  }
  .modules-grid :global(.module-card) {
    height: 100%;
    margin-bottom: 0;
    display: flex;
    flex-direction: column;
  }
  .modules-grid :global(.module-card .card-body) {
    flex: 1;
  }
  .toggle { display: flex; gap: 8px; font-size: 13px; margin-bottom: 8px; cursor: pointer; }
  .split-label { font-size: 11px; color: var(--text-muted); display: block; margin-bottom: 4px; }
  .full { width: 100%; background: var(--input-bg); border: 1px solid var(--border); border-radius: 8px; padding: 6px 10px; color: var(--text); font-size: 12px; margin-bottom: 6px; }
  .hint { font-size: 11px; color: var(--text-muted); margin: 4px 0 0; }
  .wb-row { display: flex; align-items: center; gap: 8px; margin: 6px 0; flex-wrap: wrap; }
  .wb-label { font-size: 11px; color: var(--text-muted); font-weight: 600; }
  .wb-actions { display: flex; gap: 8px; margin-bottom: 4px; }
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
