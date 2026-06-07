<script>
import { invoke } from "../api/tauri.js";
import { defaultOutputPath } from "../api/portable.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import { networkCaptureGuide } from "../guides.js";
import { getLocale, subscribeLocale } from "../stores/locale.js";

let { busy, setBusy, setMsg, timeoutPromise } = $props();
let interfaces = $state([]);
let iface = $state("");
let bpf = $state("");
let outFile = $state("");

$effect(() => {
  defaultOutputPath("capture.pcapng").then((p) => {
    if (!outFile) outFile = p;
  });
});
let maxDurationSecs = $state("3600");
let capturing = $state(false);
let packets = $state([]);
let bytesCaptured = $state(0);
let captureDuration = $state(0);
let pollId = $state(null);
let startTime = $state(null);
let locale = $state(getLocale());

$effect(() => subscribeLocale((_, resolved) => {
  locale = resolved;
}));

const text = {
  en: {
    title: "Network Capture",
    subtitle: "Packet capture via BPF filter",
    interface: "Interface",
    filterOutput: "Filter & Output",
    select: "— Select —",
    maxDuration: "Max duration (seconds):",
    infinite: "⚠ 0 = infinite (manual stop required)",
    default: "Default: 3600 (1 hour)",
    start: "Start Capture",
    stop: "Stop Capture",
    bytes: "Bytes",
    packets: "Packets",
    duration: "Duration",
    preview: "Packet Preview",
    captureStarted: "Capture started",
    captureStopped: (n) => `Capture stopped — ${n} packets previewed`,
    outputFile: "Capture output file",
    previewTime: "Time",
    previewSrc: "Src",
    previewDst: "Dst",
    previewProto: "Proto",
    previewLen: "Len",
  },
  id: {
    title: "Tangkap Jaringan",
    subtitle: "Perekaman paket melalui filter BPF",
    interface: "Antarmuka",
    filterOutput: "Filter & Keluaran",
    select: "— Pilih —",
    maxDuration: "Durasi maksimum (detik):",
    infinite: "⚠ 0 = tak terbatas (harus stop manual)",
    default: "Default: 3600 (1 jam)",
    start: "Mulai Tangkap",
    stop: "Hentikan Tangkap",
    bytes: "Byte",
    packets: "Paket",
    duration: "Durasi",
    preview: "Pratinjau Paket",
    captureStarted: "Tangkap dimulai",
    captureStopped: (n) => `Tangkap dihentikan — ${n} paket dipratinjau`,
    outputFile: "Berkas keluaran tangkapan",
    previewTime: "Waktu",
    previewSrc: "Sumber",
    previewDst: "Tujuan",
    previewProto: "Protokol",
    previewLen: "Panjang",
  },
};
function tr(key, ...args) {
  const val = text[locale]?.[key] ?? text.en[key];
  return typeof val === "function" ? val(...args) : val;
}

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
  const duration = parseInt(maxDurationSecs, 10);
  if (duration === 0) {
    setMsg(locale === "id" ? "PERINGATAN: Durasi 0 = tangkap tak terbatas — gunakan Hentikan untuk berhenti manual" : "WARN: Duration 0 = infinite capture — use Stop to end manually");
  }
  capturing = true;
  startTime = Date.now();
  packets = [];
  try {
    await timeoutPromise(
      invoke("start_network_capture", {
        interface: iface,
        bpfFilter: bpf || null,
        outputFile: outFile,
        maxDurationSecs: Number.isFinite(duration) ? duration : 3600,
      }),
      10000
    );
    setMsg(tr("captureStarted"));
    pollId = setInterval(refreshStats, 2000);
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
    capturing = false;
  }
}

async function stopCapture() {
  try {
    await invoke("cancel_network_capture");
    await refreshStats();
    setMsg(tr("captureStopped", packets.length));
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

<div class="tab-content network-tab">
  <SectionHeader title={tr("title")} subtitle={tr("subtitle")} />

  <MacCard title={tr("interface")}>
    <select bind:value={iface} class="full">
      <option value="">{tr("select")}</option>
      {#each interfaces as i}<option value={i}>{i}</option>{/each}
    </select>
  </MacCard>

  <MacCard title={tr("filterOutput")}>
    <input type="text" bind:value={bpf} placeholder={locale === "id" ? "Filter BPF (mis. bukan port 22)" : "BPF filter (e.g. not port 22)"} class="full" />
    <label for="network-output" class="sr-only">{tr("outputFile")}</label>
    <input id="network-output" type="text" bind:value={outFile} class="full" />
    <label class="duration-row">
      {tr("maxDuration")}
      <input type="number" bind:value={maxDurationSecs} min="0" placeholder="3600" />
      <span class="hint">{parseInt(maxDurationSecs, 10) === 0 ? tr("infinite") : tr("default")}</span>
    </label>
  </MacCard>

  {#if !capturing}
    <button onclick={startCapture} class="btn-primary" disabled={!iface}>{tr("start")}</button>
  {:else}
    <button onclick={stopCapture} class="btn-danger">{tr("stop")}</button>
  {/if}

  <div class="stats-row">
    <MacCard title={tr("bytes")}><span class="stat">{(bytesCaptured / 1024).toFixed(1)} KB</span></MacCard>
    <MacCard title={tr("packets")}><span class="stat">{packets.length}</span></MacCard>
    <MacCard title={tr("duration")}><span class="stat">{captureDuration}s</span></MacCard>
  </div>

  {#if packets.length}
    <MacCard title={tr("preview")}>
      <div class="table-wrap">
        <table>
        <thead><tr><th>#</th><th>{tr("previewTime")}</th><th>{tr("previewSrc")}</th><th>{tr("previewDst")}</th><th>{tr("previewProto")}</th><th>{tr("previewLen")}</th></tr></thead>
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
  .full { width: 100%; background: var(--input-bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; font-size: 13px; margin-bottom: 8px; }
  .duration-row { display: flex; flex-direction: column; gap: 4px; font-size: 13px; margin-top: 4px; }
  .duration-row input { width: 120px; }
  .hint { font-size: 11px; color: var(--warn); }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
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
