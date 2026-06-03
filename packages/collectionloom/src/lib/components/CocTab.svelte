<script>
import { invoke } from "@tauri-apps/api/core";
import GuideCard from "./GuideCard.svelte";
import { snapshotGuide } from "../guides.js";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();
let evidenceId = $state("");
let caseName = $state("");
let operator = $state("Yusuf Shalahuddin");
let device = $state("");
let actions = $state([]);
let signature = $state("");
let publicKey = $state("");
let signLoading = $state(false);

// Auto-generate Evidence ID on mount
$effect(() => {
  generateEvidenceId();
});

async function generateEvidenceId() {
  try {
    const id = await timeoutPromise(invoke("generate_evidence_id"), 5000);
    if (id) evidenceId = id;
  } catch(e) {
    // Silently fail — user can create CoC manually
  }
}

async function createCoc() {
  setBusy(true);
  try {
    evidenceId = await timeoutPromise(invoke("create_chain_of_custody", { caseName, operator, sourceDevice: device }), 5000);
    actions = [{ timestamp: new Date().toISOString(), action: "CoC created", details: `Evidence ${evidenceId}`, hash: null }];
    setMsg(`✅ Chain of custody created: ${evidenceId}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function addAction(act, det) {
  actions = [...actions, { timestamp: new Date().toISOString(), action: act, details: det, hash: null }];
}
async function generatePdf() {
  setBusy(true);
  try {
    const path = await timeoutPromise(invoke("generate_coc_report", { evidenceId }), 15000);
    setMsg(`✅ PDF report saved to ${path}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}

async function signCoc() {
  if (!evidenceId) return;
  signLoading = true;
  try {
    const result = await timeoutPromise(invoke("sign_coc", { evidenceId }), 10000);
    signature = result.signature || result;
    publicKey = result.public_key || "";
    setMsg(`✅ CoC signed successfully`);
  } catch(e) {
    setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
  }
  signLoading = false;
}
</script>

<div>
  <h3>📋 Chain of Custody</h3>

  <!-- Evidence ID display -->
  {#if evidenceId}
    <div class="evidence-banner">
      <span class="evidence-label">Evidence ID:</span>
      <span class="evidence-value">{evidenceId}</span>
    </div>
  {/if}

  <!-- QR Code placeholder -->
  {#if evidenceId}
    <div class="qr-section">
      <div class="qr-box">
        <div class="qr-ascii">
          <span class="qr-label">QR</span>
          <pre class="qr-art">
███████████████████████
██                 ██
██  ████  ██  ████  ██
██  ████  ██  ████  ██
██  ████  ██  ████  ██
██                 ██
███████████████████████
          </pre>
        </div>
        <div class="qr-id">
          <span class="qr-id-label">Evidence ID</span>
          <span class="qr-id-value">{evidenceId}</span>
        </div>
      </div>
    </div>
  {/if}

  <div class="row"><label>Case: <input type="text" bind:value={caseName} /></label></div>
  <div class="row"><label>Operator: <input type="text" bind:value={operator} /></label></div>
  <div class="row"><label>Source Device: <input type="text" bind:value={device} placeholder="/dev/sda" /></label></div>
  
  {#if !evidenceId}
    <button onclick={createCoc} class="btn-primary" disabled={!caseName||!device}>📋 Create Chain of Custody</button>
  {:else}
    <div class="evidence-id">Evidence ID: <strong>{evidenceId}</strong></div>
    
    <div class="actions-log">
      <h4>Action Log</h4>
      {#each actions as a, i}
        <div class="log-entry">
          <span class="time">{a.timestamp}</span>
          <span class="action">{a.action}</span>
          <span class="detail">{a.details}</span>
        </div>
      {/each}
    </div>

    <div class="add-action">
      <input type="text" placeholder="Action (e.g. imaging_start)" id="newAction" />
      <input type="text" placeholder="Details" id="newDetails" />
      <button onclick={() => { let a=document.getElementById('newAction').value; let d=document.getElementById('newDetails').value; addAction(a,d); }} class="btn-sm">+ Add</button>
    </div>

    <div class="btn-row">
      <button onclick={generatePdf} class="btn-primary">📄 Generate PDF Report</button>
      <button onclick={signCoc} class="btn-sign" disabled={signLoading || !evidenceId}>
        {signLoading ? "⏳ Signing..." : "✍️ Sign with Ed25519"}
      </button>
    </div>

    {#if signature}
      <div class="signature-section">
        <div class="sig-row">
          <span class="sig-label">Signature</span>
          <code class="sig-value">{signature}</code>
        </div>
        {#if publicKey}
          <div class="sig-row">
            <span class="sig-label">Public Key</span>
            <code class="sig-value">{publicKey}</code>
          </div>
        {/if}
      </div>
    {/if}
  {/if}

  <!-- GuideCard -->
  <div style="margin-top:16px">
    <GuideCard title={snapshotGuide.title} icon={snapshotGuide.icon} steps={snapshotGuide.steps} references={snapshotGuide.references} />
  </div>
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }

.evidence-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: rgba(34,197,94,0.08);
  border: 1px solid var(--success);
  border-radius: 8px;
  margin-bottom: 12px;
}
.evidence-label {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.evidence-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--success);
  font-family: var(--mono, monospace);
}

.qr-section {
  margin-bottom: 16px;
}
.qr-box {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background: #111;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.qr-ascii {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.qr-label {
  font-size: 10px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 1px;
}
.qr-art {
  margin: 0;
  font-size: 6px;
  line-height: 1.1;
  color: #22c55e;
  font-family: monospace;
}
.qr-id {
  margin-top: 8px;
  text-align: center;
}
.qr-id-label {
  display: block;
  font-size: 9px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.qr-id-value {
  font-size: 11px;
  font-weight: 600;
  color: #e0e0e0;
  font-family: var(--mono, monospace);
}

.row { margin-bottom:10px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; width:300px; }
.btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary:disabled { opacity:0.5; }
.btn-sm { padding:4px 10px; background:var(--border); color:#e0e0e0; border:none; border-radius:4px; cursor:pointer; font-size:11px; }
.evidence-id { padding:10px; background:#1a2e1a; border:1px solid var(--success); border-radius:8px; margin:12px 0; }
.actions-log { margin:16px 0; max-height:200px; overflow-y:auto; }
h4 { font-size:13px; margin:0 0 8px; }
.log-entry { display:flex; gap:10px; padding:4px 0; font-size:11px; border-bottom:1px solid var(--border); }
.time { color:var(--text-secondary); white-space:nowrap; }
.action { font-weight:600; min-width:120px; }
.add-action { display:flex; gap:6px; margin-top:10px; }
.add-action input { width:150px; }

.btn-row {
  display: flex;
  gap: 10px;
  margin-top: 16px;
  flex-wrap: wrap;
}
.btn-sign {
  padding: 10px 24px;
  background: rgba(34,197,94,0.15);
  color: #22c55e;
  border: 1px solid #22c55e;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  font-size: 13px;
  transition: background 0.15s;
}
.btn-sign:hover:not(:disabled) {
  background: rgba(34,197,94,0.25);
}
.btn-sign:disabled {
  opacity: 0.5;
  cursor: default;
}

.signature-section {
  margin-top: 14px;
  padding: 12px;
  background: #1a1a1a;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.sig-row {
  margin-bottom: 8px;
}
.sig-row:last-child {
  margin-bottom: 0;
}
.sig-label {
  display: block;
  font-size: 10px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}
.sig-value {
  display: block;
  font-size: 11px;
  color: #22c55e;
  font-family: var(--mono, monospace);
  word-break: break-all;
  padding: 4px 8px;
  background: #111;
  border-radius: 4px;
  line-height: 1.4;
}
</style>
