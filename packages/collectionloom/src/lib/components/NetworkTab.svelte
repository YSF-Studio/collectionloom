<script>
import { invoke } from "@tauri-apps/api/core";
import GuideCard from "./GuideCard.svelte";
import { networkCaptureGuide } from "../guides.js";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();
let interfaces = $state([]);
let iface = $state("");
let bpf = $state("");
let outFile = $state("/mnt/evidence/capture.pcapng");
let capturing = $state(false);
let totalPackets = $state(0);

// Packet preview table (placeholder)
let samplePackets = $state([
  { no: 1, time: "0.000000", src: "192.168.1.100", dst: "192.168.1.1", proto: "TCP", len: 74 },
  { no: 2, time: "0.001234", src: "192.168.1.1", dst: "192.168.1.100", proto: "TCP", len: 66 },
  { no: 3, time: "0.002345", src: "10.0.0.5", dst: "10.0.0.1", proto: "UDP", len: 482 },
  { no: 4, time: "0.005678", src: "10.0.0.1", dst: "10.0.0.5", proto: "DNS", len: 89 },
  { no: 5, time: "0.008901", src: "192.168.1.100", dst: "8.8.8.8", proto: "ICMP", len: 98 },
]);

// Capture stats
let bytesCaptured = $state(0);
let packetsCount = $state(0);
let captureDuration = $state(0);

// Ring buffer info
let ringFileSize = $state("0 B");
let rotationStatus = $state("Inactive");

async function listIface() {
  setBusy(true);
  try { interfaces = await timeoutPromise(invoke("list_interfaces"), 5000); } catch(e) {}
  setBusy(false);
}
async function startCapture() {
  capturing = true;
  try {
    const r = await timeoutPromise(invoke("start_network_capture", { interface: iface, bpfFilter: bpf || null, outputFile: outFile }), 10000);
    setMsg(r);
    bytesCaptured = 0;
    packetsCount = 0;
    captureDuration = 0;
    totalPackets = 0;
    ringFileSize = "0 B";
    rotationStatus = "Active";
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    setMsg(`❌ ${err}`);
  }
  capturing = false;
}
async function stopCapture() {
  try {
    const r = await invoke("cancel_network_capture");
    totalPackets = packetsCount;
    rotationStatus = "Stopped";
  } catch(e) {}
  capturing = false;
}
$effect(() => { listIface(); });
</script>

<div>
  <h3>🌐 Network Capture</h3>
  <div class="row"><label>Interface: <select bind:value={iface}><option value="">-- Select --</option>{#each interfaces as i}<option value={i}>{i}</option>{/each}</select></label></div>
  <div class="row"><label>BPF Filter: <input type="text" bind:value={bpf} placeholder="not port 22" /></label></div>
  <div class="row"><label>Output: <input type="text" bind:value={outFile} /></label></div>
  {#if !capturing}
    <button onclick={startCapture} class="btn-primary" disabled={!iface}>▶ Start Capture</button>
  {:else}
    <button onclick={stopCapture} class="btn-danger">■ Stop</button>
  {/if}

  {#if capturing || totalPackets > 0}
  <div class="section">
    <h4>📦 Packet Preview</h4>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>No</th>
            <th>Time</th>
            <th>Source</th>
            <th>Dest</th>
            <th>Proto</th>
            <th>Len</th>
          </tr>
        </thead>
        <tbody>
          {#if capturing}
            {#each samplePackets as pkt}
            <tr>
              <td>{pkt.no}</td>
              <td>{pkt.time}</td>
              <td>{pkt.src}</td>
              <td>{pkt.dst}</td>
              <td>{pkt.proto}</td>
              <td>{pkt.len}</td>
            </tr>
            {/each}
          {:else if totalPackets > 0}
            <tr><td colspan="6" class="done-msg">Capture complete — {totalPackets} total packets</td></tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
  {/if}

  <div class="stats-row">
    <div class="stat-card">
      <span class="stat-label">Bytes</span>
      <span class="stat-value">{bytesCaptured > 0 ? (bytesCaptured / 1024).toFixed(1) + " KB" : "—"}</span>
    </div>
    <div class="stat-card">
      <span class="stat-label">Packets</span>
      <span class="stat-value">{packetsCount > 0 ? packetsCount : "—"}</span>
    </div>
    <div class="stat-card">
      <span class="stat-label">Duration</span>
      <span class="stat-value">{captureDuration > 0 ? captureDuration + "s" : "—"}</span>
    </div>
  </div>

  <div class="ring-info">
    <span class="ring-label">Ring Buffer:</span>
    <span class="ring-value">{ringFileSize}</span>
    <span class="ring-badge" class:active={rotationStatus === "Active"}>{rotationStatus}</span>
  </div>

  <GuideCard title={networkCaptureGuide.title} icon={networkCaptureGuide.icon} steps={networkCaptureGuide.steps} references={networkCaptureGuide.references} />
</div>
<style>
h3 { margin:0 0 16px; font-size:16px; }
h4 { margin:0 0 10px; font-size:14px; color:var(--text-secondary); }
.row { margin-bottom:10px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input, select { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; width:100%; }
.btn-primary, .btn-danger { padding:10px 24px; color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary { background:var(--primary); }
.btn-danger { background:var(--danger); }
.section { margin-top:16px; }
.table-wrap { overflow-x:auto; border:1px solid var(--border); border-radius:8px; }
table { width:100%; border-collapse:collapse; font-size:12px; }
th { background:#1a1a1a; color:var(--text-secondary); padding:6px 10px; text-align:left; font-weight:600; border-bottom:1px solid var(--border); }
td { padding:5px 10px; border-bottom:1px solid rgba(255,255,255,0.04); color:#ccc; font-family:var(--mono); }
.done-msg { text-align:center; color:var(--primary); padding:16px; font-weight:600; }
.stats-row { display:flex; gap:10px; margin-top:12px; }
.stat-card { flex:1; background:#1a1a1a; border:1px solid var(--border); border-radius:8px; padding:10px; display:flex; flex-direction:column; gap:2px; }
.stat-label { font-size:10px; color:var(--text-muted); text-transform:uppercase; letter-spacing:0.5px; }
.stat-value { font-size:14px; font-weight:600; color:var(--text); }
.ring-info { display:flex; align-items:center; gap:8px; margin-top:12px; font-size:12px; }
.ring-label { color:var(--text-muted); }
.ring-value { font-family:var(--mono); color:var(--text-secondary); }
.ring-badge { padding:2px 8px; border-radius:4px; font-size:10px; font-weight:600; background:#2a2a2a; color:var(--text-muted); }
.ring-badge.active { background:rgba(34,197,94,0.15); color:var(--success); }
</style>
