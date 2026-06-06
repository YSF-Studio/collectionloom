<script>
import { invoke } from "../api/tauri.js";
import GuideCard from "./GuideCard.svelte";
import MacCard from "./ui/MacCard.svelte";
import SectionHeader from "./ui/SectionHeader.svelte";
import { snapshotGuide } from "../guides.js";
import { createCase } from "../api/case.js";

let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();
let evidenceId = $state("");
let caseName = $state("");
let operator = $state("");
let device = $state("");
let timezone = $state(Intl.DateTimeFormat().resolvedOptions().timeZone);
let actions = $state([]);
let signature = $state("");
let publicKey = $state("");
let signLoading = $state(false);
let qrDataUrl = $state("");
let linkedCaseId = $state("");

$effect(() => {
  generateEvidenceId();
  if (!operator) {
    operator = typeof navigator !== "undefined" ? navigator.userAgent.includes("Mac") ? "Investigator" : "Investigator" : "Investigator";
  }
});

async function generateEvidenceId() {
  try {
    const id = await timeoutPromise(invoke("generate_evidence_id"), 5000);
    if (id) evidenceId = id;
  } catch {
    /* silent */
  }
}

function bytesToDataUrl(bytes) {
  const arr = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  if (arr.length >= 8 && arr[0] === 0x89 && arr[1] === 0x50) {
    let binary = "";
    for (let i = 0; i < arr.length; i++) binary += String.fromCharCode(arr[i]);
    return `data:image/png;base64,${btoa(binary)}`;
  }
  return "";
}

async function loadQr() {
  if (!evidenceId) return;
  try {
    const bytes = await invoke("generate_qr_label", {
      evidenceId,
      device: device || "unknown",
      caseName: caseName || "case",
    });
    qrDataUrl = bytesToDataUrl(new Uint8Array(bytes));
  } catch {
    qrDataUrl = "";
  }
}

$effect(() => {
  if (evidenceId && caseName) loadQr();
});

async function createCoc() {
  setBusy(true);
  try {
    const linkedCase = await createCase({
      title: caseName,
      operator,
      timezone,
      purpose: "Chain of custody",
      description: `Evidence for ${device}`,
    });
    linkedCaseId = linkedCase.case_id;
    sharedState.caseId = linkedCaseId;

    evidenceId = await timeoutPromise(
      invoke("create_chain_of_custody", { caseName, operator, sourceDevice: device }),
      5000
    );
    actions = [
      {
        timestamp: new Date().toISOString(),
        action: "CoC created",
        details: `Evidence ${evidenceId} · Case ${linkedCaseId.slice(0, 8)}…`,
      },
    ];
    await loadQr();
    setMsg(`OK: Chain of custody created: ${evidenceId}`);
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function addAction(act, det) {
  actions = [...actions, { timestamp: new Date().toISOString(), action: act, details: det }];
}

async function generatePdf() {
  setBusy(true);
  try {
    const path = await timeoutPromise(invoke("generate_coc_report", { evidenceId }), 15000);
    setMsg(`OK: PDF report saved to ${path}`);
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  setBusy(false);
}

async function signCoc() {
  if (!evidenceId) return;
  signLoading = true;
  try {
    const result = await timeoutPromise(invoke("sign_coc", { evidenceId }), 10000);
    signature = result.signature_hex || result.signature || "";
    publicKey = result.public_key_hex || result.public_key || "";
    setMsg("OK: CoC signed successfully");
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  signLoading = false;
}
</script>

<div class="coc-tab">
  <SectionHeader title="Chain of Custody" subtitle="Evidence tracking with Ed25519 signatures" />

  {#if evidenceId}
    <MacCard title="Evidence ID">
      <code class="evidence-id">{evidenceId}</code>
      {#if linkedCaseId}<p class="case-link">Case: <code>{linkedCaseId}</code></p>{/if}
    </MacCard>
  {/if}

  {#if evidenceId}
    <MacCard title="QR Label">
      <div class="qr-wrap">
        {#if qrDataUrl}
          <img class="qr-img" src={qrDataUrl} alt="Evidence QR label for {evidenceId}" />
        {:else}
          <span class="qr-loading">Generating QR…</span>
        {/if}
        <span class="qr-id">{evidenceId}</span>
      </div>
    </MacCard>
  {/if}

  <MacCard title="Case Details">
    <div class="field"><label for="coc-title">Case Title</label><input id="coc-title" type="text" bind:value={caseName} /></div>
    <div class="field"><label for="coc-operator">Operator</label><input id="coc-operator" type="text" bind:value={operator} /></div>
    <div class="field"><label for="coc-tz">Timezone</label><input id="coc-tz" type="text" bind:value={timezone} /></div>
    <div class="field"><label for="coc-device">Source Device</label><input id="coc-device" type="text" bind:value={device} placeholder="/dev/disk2" /></div>
  </MacCard>

  {#if !linkedCaseId}
    <button onclick={createCoc} class="btn-primary" disabled={!caseName || !device || busy}>
      Create Chain of Custody
    </button>
  {:else}
    <MacCard title="Action Log">
      {#each actions as a}
        <div class="log-entry">
          <span class="time">{a.timestamp}</span>
          <span class="action">{a.action}</span>
          <span class="detail">{a.details}</span>
        </div>
      {/each}
    </MacCard>

    <div class="actions">
      <button onclick={() => addAction("Imaging started", device)} class="btn-sm">Log Imaging</button>
      <button onclick={() => addAction("Transferred", "Secure storage")} class="btn-sm">Log Transfer</button>
      <button onclick={generatePdf} class="btn-sm">Generate PDF</button>
      <button onclick={signCoc} class="btn-sm" disabled={signLoading}>{signLoading ? "Signing…" : "Sign CoC"}</button>
    </div>

    {#if signature}
      <MacCard title="Signature">
        <div class="sig-row"><span>Signature</span><code>{signature.slice(0, 48)}…</code></div>
        {#if publicKey}<div class="sig-row"><span>Public Key</span><code>{publicKey.slice(0, 48)}…</code></div>{/if}
      </MacCard>
    {/if}
  {/if}

  <GuideCard title={snapshotGuide.title} icon={snapshotGuide.icon} steps={snapshotGuide.steps} references={snapshotGuide.references} />
</div>

<style>
  .coc-tab { max-width: 640px; }
  .evidence-id { font-family: var(--mono); font-size: 14px; color: var(--primary); }
  .case-link { margin: 8px 0 0; font-size: 12px; color: var(--text-secondary); }
  .qr-wrap { display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .qr-img { width: 160px; height: 160px; image-rendering: pixelated; border-radius: 8px; border: 1px solid var(--border); background: white; padding: 8px; }
  .qr-loading { font-size: 12px; color: var(--text-muted); }
  .qr-id { font-family: var(--mono); font-size: 11px; color: var(--text-secondary); }
  .field { margin-bottom: 10px; }
  .field label { display: block; font-size: 11px; color: var(--text-muted); margin-bottom: 4px; }
  .field input { width: 100%; background: var(--input-bg); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; color: var(--text); font-size: 13px; }
  .btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 10px; font-weight: 600; cursor: pointer; margin-bottom: 16px; }
  .btn-primary:disabled { opacity: 0.5; }
  .btn-sm { padding: 6px 12px; background: var(--btn-secondary-bg); border: 1px solid var(--border); border-radius: 6px; color: var(--btn-secondary-text); cursor: pointer; font-size: 12px; margin-right: 6px; }
  .actions { margin: 12px 0; }
  .log-entry { display: flex; gap: 10px; font-size: 11px; padding: 4px 0; border-bottom: 1px solid rgba(255,255,255,0.04); }
  .time { color: var(--text-muted); font-family: var(--mono); }
  .action { font-weight: 600; }
  .detail { color: var(--text-secondary); }
  .sig-row { display: flex; gap: 10px; font-size: 11px; margin-bottom: 6px; }
  .sig-row span { color: var(--text-muted); min-width: 80px; }
  code { font-family: var(--mono); word-break: break-all; }
</style>
