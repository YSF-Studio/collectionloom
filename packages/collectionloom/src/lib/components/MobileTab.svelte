<script>
import { invoke } from "../api/tauri.js";
import { defaultOutputPath } from "../api/portable.js";
import GuideCard from "./GuideCard.svelte";
import { mobileTriageGuide } from "../guides.js";
let { busy, setBusy, setMsg, timeoutPromise } = $props();
let androidDevices = $state([]);
let iosDevices = $state([]);
let outputPath = $state("");

$effect(() => {
  defaultOutputPath("mobile_backup.ab").then((p) => {
    if (!outputPath) outputPath = p;
  });
});
let androidBackupProgress = $state({});
let iosBackupProgress = $state({});

async function scanAndroid() {
  setBusy(true);
  androidDevices = [];
  try { androidDevices = await timeoutPromise(invoke("list_android_devices"), 10000); } catch(e) { setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function scanIos() {
  setBusy(true);
  iosDevices = [];
  try { iosDevices = await timeoutPromise(invoke("list_ios_devices"), 10000); } catch(e) { setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function backupAndroid(id) {
  setBusy(true);
  androidBackupProgress[id] = "Backing up...";
  try {
    const r = await timeoutPromise(invoke("adb_backup", { deviceId: id, output: outputPath }), 300000);
    setMsg(`OK: ${r}`);
    androidBackupProgress[id] = "✅ Complete";
  } catch(e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
    androidBackupProgress[id] = `❌ ${typeof e === "string" ? e : String(e)}`;
  }
  setBusy(false);
}
async function backupIos(id) {
  setBusy(true);
  iosBackupProgress[id] = "Backing up...";
  try {
    const r = await timeoutPromise(invoke("ios_backup", { deviceId: id, output: outputPath }), 300000);
    setMsg(`OK: ${r}`);
    iosBackupProgress[id] = "✅ Complete";
  } catch(e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
    iosBackupProgress[id] = `❌ ${typeof e === "string" ? e : String(e)}`;
  }
  setBusy(false);
}
</script>

<div>
  <h3>Mobile Triage</h3>
  <div class="row">
    <label for="mobile-output">Backup destination:</label>
    <input id="mobile-output" type="text" bind:value={outputPath} disabled={busy} placeholder="/tmp/mobile_backup.ab" />
  </div>
  <div class="cols">
    <div class="col">
      <h4>Android</h4>
      <button onclick={scanAndroid} disabled={busy} class="btn-sm">Scan ADB</button>
      {#each androidDevices as d}
        <div class="device">
          <span>{d.model} ({d.id})</span>
          <span>
            <button onclick={() => backupAndroid(d.id)} class="btn-sm" disabled={busy || !outputPath}>Backup</button>
            {#if androidBackupProgress[d.id]}
              <span class="progress-label">{androidBackupProgress[d.id]}</span>
            {/if}
          </span>
        </div>
      {/each}
    </div>
    <div class="col">
      <h4>iOS</h4>
      <button onclick={scanIos} disabled={busy} class="btn-sm">Scan idevice</button>
      {#each iosDevices as d}
        <div class="device">
          <span>{d.model} ({d.id})</span>
          <span>
            <button onclick={() => backupIos(d.id)} class="btn-sm" disabled={busy || !outputPath}>Backup</button>
            {#if iosBackupProgress[d.id]}
              <span class="progress-label">{iosBackupProgress[d.id]}</span>
            {/if}
          </span>
        </div>
      {/each}
    </div>
  </div>
  <p class="note">Note: Faraday bag reminder: isolate mobile devices before acquisition</p>

  <GuideCard title={mobileTriageGuide.title} icon={mobileTriageGuide.icon} steps={mobileTriageGuide.steps} references={mobileTriageGuide.references} />
</div>
<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { display:flex; gap:10px; align-items:center; margin-bottom:12px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input { background: var(--input-bg); color: var(--text); border:1px solid var(--border); border-radius:6px; padding:6px 10px; font-size:13px; width:100%; }
.cols { display:grid; grid-template-columns:1fr 1fr; gap:20px; }
.col { background: var(--input-bg); border:1px solid var(--border); border-radius:8px; padding:12px; }
h4 { margin:0 0 8px; font-size:13px; }
.device { font-size:12px; padding:6px 0; border-bottom:1px solid var(--border); display:flex; justify-content:space-between; align-items:center; gap:8px; }
.device span { display:flex; align-items:center; gap:4px; }
.btn-sm { padding:4px 8px; background:var(--border); color:#e0e0e0; border:none; border-radius:4px; cursor:pointer; font-size:11px; }
.btn-sm:disabled { opacity:0.5; cursor:not-allowed; }
.note { font-size:11px; color:var(--warn); margin-top:16px; }
.progress-label { font-size:10px; color:var(--text-secondary); }
</style>
