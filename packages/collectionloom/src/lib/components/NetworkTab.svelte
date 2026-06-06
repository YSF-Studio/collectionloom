<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import { networkCaptureGuide } from "../guides.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();
let interfaces = $state([]);
let iface = $state("");
let bpf = $state("");
let outFile = $state("/tmp/evidence/capture.pcapng");
let capturing = $state(false);
let packets = $state([]);
let bytesCaptured = $state(0);
let captureDuration = $state(0);
let pollId = $state(null);
let startTime = $state(null);

async function listIface() {
  setBusy(true);
  try {
    interfaces = await timeoutPromise(invoke("list_interfaces"), 5000);
  } catch {
    /* ignore */
  }
  setBusy(false);
}

async function refreshStats() {
  try {
    const stats = await invoke("get_capture_stats", { outputFile: outFile });
    bytesCaptured = stats.bytes_captured || 0;
    if (startTime) captureDuration = Math.round((Date.now() - startTime) / 1000);

    const pkts = await invoke("get_capture_packets", { outputFile: outFile, limit: 10 });
    if (pkts?.length) packets = pkts;
  } catch {
    /* ignore */
  }
}

async function startCapture() {
  capturing = true;
  startTime = Date.now();
  packets = [];
  try {
    await timeoutPromise(
      invoke("start_network_capture", { interface: iface, bpfFilter: bpf || null, outputFile: outFile }),
      10000
    );
    setMsg("Capture started");
    pollId = setInterval(refreshStats, 2000);
  } catch (e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
    capturing = false;
  }
}

async function stopCapture() {
  try {
    await invoke("cancel_network_capture");
    await refreshStats();
    setMsg(`Capture stopped — ${packets.length} packets previewed`);
  } catch {
    /* ignore */
  }
  if (pollId) clearInterval(pollId);
  capturing = false;
  startTime = null;
}

$effect(() => {
  listIface();
});
</script>

<div class="network-tab">
  <SectionHeader title="Network Capture" subtitle="Packet capture via BPF filter" />

  <MacCard title="Interface">
    <select bind:value={iface} class="full">
      <option value="">— Select —</option>
      {#each interfaces as i}<option value={i}>{i}</option>{/each}
    </select>
  </MacCard>

  <MacCard title="Filter & Output">
    <input type="text" bind:value={bpf} placeholder="BPF filter (e.g. not port 22)" class="full" />
    <input type="text" bind:value={outFile} class="full" />
  </MacCard>

  {#if !capturing}
    <button onclick={startCapture} class="btn-primary" disabled={!iface}>Start Capture</button>
  {:else}
    <button onclick={stopCapture} class="btn-danger">Stop Capture</button>
  {/if}

  <div class="stats-row">
    <MacCard title="Bytes"><span class="stat">{(bytesCaptured / 1024).toFixed(1)} KB</span></MacCard>
    <MacCard title="Packets"><span class="stat">{packets.length}</span></MacCard>
    <MacCard title="Duration"><span class="stat">{captureDuration}s</span></MacCard>
  </div>

  {#if packets.length}
    <MacCard title="Packet Preview">
      <div class="table-wrap">
        <table>
          <thead><tr><th>#</th><th>Time</th><th>Src</th><th>Dst</th><th>Proto</th><th>Len</th></tr></thead>
          <tbody>
            {#each packets as pkt}
              <tr>
                <td>{pkt.no}</td>
                <td>{pkt.time || "—"}</td>
                <td>{pkt.src || "—"}</td>
                <td>{pkt.dst || "—"}</td>
                <td>{pkt.proto || "—"}</td>
                <td>{pkt.len || "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </MacCard>
  {/if}

  <GuideCard title={networkCaptureGuide.title} icon={networkCaptureGuide.icon} steps={networkCaptureGuide.steps} references={networkCaptureGuide.references} />
</div>

<style>
  .network-tab { max-width: 720px; }
  .full { width: 100%; background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; font-size: 13px; margin-bottom: 8px; }
  .btn-primary, .btn-danger { padding: 10px 24px; color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; margin-bottom: 12px; }
  .btn-primary { background: var(--primary); }
  .btn-danger { background: var(--danger); }
  .stats-row { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-bottom: 12px; }
  .stat { font-size: 16px; font-weight: 600; }
  .table-wrap { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 11px; font-family: var(--mono); }
  th { text-align: left; color: var(--text-muted); padding: 4px 8px; border-bottom: 1px solid var(--border); }
  td { padding: 4px 8px; border-bottom: 1px solid rgba(255,255,255,0.04); }
</style>
