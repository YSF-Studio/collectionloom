<script>
  import { invoke } from "../api/tauri.js";
  import GuideCard from "./GuideCard.svelte";
  import { cloudEvidenceGuide } from "../guides.js";
  import { getResolvedLocale, subscribeLocale } from "../stores/locale.js";

  let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();

  let provider = $state("aws");
  let region = $state("us-east-1");
  let resourceId = $state("");
  let credentialPath = $state("");
  let collBusy = $state(false);
  let msg = $state("");
  let result = $state(null);
  let resultRaw = $state("");
  let snapshotId = $state("");
  let locale = $state(getResolvedLocale());

  const text = {
    en: {
      title: "Cloud Snapshot",
      note: "Credentials loaded via native file picker — never stored in browser memory",
      provider: "Provider:",
      region: "Region:",
      resource: "Resource ID:",
      credentials: "Credentials file:",
      browse: "Browse…",
      jsonHint: 'JSON: {"access_key","secret_key"} or AWS INI format. Leave empty to pick at snapshot time.',
      required: "Resource ID is required",
      create: "Create Snapshot",
      creating: "Creating Snapshot...",
      requestSent: "Snapshot Request Sent",
      providerLabel: "Provider:",
      snapshotId: "Snapshot ID:",
      contacting: "Contacting cloud provider... (may take 15-30s)",
    },
    id: {
      title: "Snapshot Cloud",
      note: "Kredensial dimuat lewat native file picker — tidak pernah disimpan di memori browser",
      provider: "Penyedia:",
      region: "Region:",
      resource: "Resource ID:",
      credentials: "File kredensial:",
      browse: "Telusuri…",
      jsonHint: 'JSON: {"access_key","secret_key"} atau format AWS INI. Biarkan kosong untuk dipilih saat snapshot.',
      required: "Resource ID wajib diisi",
      create: "Buat Snapshot",
      creating: "Membuat Snapshot...",
      requestSent: "Permintaan Snapshot Terkirim",
      providerLabel: "Penyedia:",
      snapshotId: "Snapshot ID:",
      contacting: "Menghubungi penyedia cloud... (mungkin 15-30 detik)",
    },
  };

  const t = (key) => text[locale]?.[key] ?? text.en[key] ?? key;

  const unsubscribe = subscribeLocale((_, resolved) => {
    locale = resolved;
  });

  $effect(() => () => unsubscribe());

  $effect(() => {
    if (msg && !msg.startsWith("ERR:")) {
      const t = setTimeout(() => msg = "", 8000);
      return () => clearTimeout(t);
    }
  });

  function placeholderText() {
    if (provider === "aws") return "vol-xxxxxxxxxxxx";
    if (provider === "azure") return "sub-id|rg-name|disk-name";
    if (provider === "gcp") return "project-id|zone|disk-name";
    if (provider === "alibaba") return "d-xxxxxxxxxxxxx";
    return "";
  }

  function providerHint() {
    if (provider === "aws") return "AWS: Volume ID (vol-...)";
    if (provider === "azure") return "Azure: subscription|resourceGroup|diskName";
    if (provider === "gcp") return "GCP: project|zone|diskName";
    if (provider === "alibaba") return "Alibaba: Disk ID (d-...)";
    return "";
  }

  async function pickCredentials() {
    try {
      const path = await invoke("pick_cloud_credentials");
      if (path) credentialPath = path;
    } catch (e) {
      msg = `ERR: ${typeof e === "string" ? e : String(e)}`;
    }
  }

  async function doCreateSnapshot() {
    if (!resourceId) {
      msg = `ERR: ${t("required")}`;
      return;
    }
    setBusy(true);
    collBusy = true;
    result = null;
    resultRaw = "";
    snapshotId = "";
    msg = "";
    try {
      const res = await timeoutPromise(invoke("create_cloud_snapshot", {
        provider,
        region,
        resourceId,
        credentialPath: credentialPath || null,
      }), 60000);
      result = res;
      resultRaw = JSON.stringify(res, null, 2);
      if (res && typeof res === "object") {
        if (res.snapshot_id) snapshotId = res.snapshot_id;
        else if (res.SnapshotId) snapshotId = res.SnapshotId;
        else if (res.id) snapshotId = res.id;
        else if (res.response && typeof res.response === "string" && res.response.includes("<snapshotId>")) {
          const m = res.response.match(/<snapshotId>([^<]+)<\/snapshotId>/);
          if (m) snapshotId = m[1];
        }
      }
      if (!snapshotId && typeof res === "string") {
        const m = res.match(/snap-[a-z0-9]+/i);
        if (m) snapshotId = m[0];
      }
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      if (err.includes("<?xml") || err.includes("<?XML") || err.includes("<Create")) {
        resultRaw = err;
        result = { provider, response: "(see raw response below)" };
        const m = err.match(/snap-[a-z0-9]+/i);
        if (m) snapshotId = m[0];
      } else {
        msg = `ERR: ${err}`;
      }
    }
    setBusy(false);
    collBusy = false;
  }
</script>

<div>
  <h3>{t("title")}</h3>
  <p class="note">{t("note")}</p>

  <div class="row">
    <label>{t("provider")}
      <select bind:value={provider} disabled={collBusy}>
        <option value="aws">AWS — {locale === "id" ? "Buat Snapshot EBS" : "Create EBS Snapshot"}</option>
        <option value="azure">Azure — {locale === "id" ? "Buat Snapshot Disk" : "Create Disk Snapshot"}</option>
        <option value="gcp">GCP — {locale === "id" ? "Buat Snapshot Persistent Disk" : "Create Persistent Disk Snapshot"}</option>
        <option value="alibaba">Alibaba — {locale === "id" ? "Buat Snapshot Disk" : "Create Disk Snapshot"}</option>
      </select>
    </label>
  </div>

  <div class="row">
    <label>{t("region")}
      <input type="text" bind:value={region} disabled={collBusy} />
    </label>
  </div>

  <div class="row">
    <label>{t("resource")}
      <input type="text" bind:value={resourceId} disabled={collBusy} placeholder={placeholderText()} />
    </label>
    <span class="hint">{providerHint()}</span>
  </div>

  <div class="row cred-row">
    <label>{t("credentials")}
      <input type="text" bind:value={credentialPath} disabled={collBusy} placeholder={locale === "id" ? "Pilih file kredensial JSON/INI…" : "Select JSON/INI credentials file…"} readonly />
    </label>
    <button class="btn-sm" onclick={pickCredentials} disabled={collBusy}>{t("browse")}</button>
  </div>
  <p class="hint">{t("jsonHint")}</p>

  {#if msg}
    <div class="result-card" class:error={msg.startsWith("ERR:")}>{msg}</div>
  {/if}

  <button class="btn-primary" onclick={doCreateSnapshot} disabled={collBusy}>
    {collBusy ? t("creating") : t("create")}
  </button>

  {#if result}
    <div class="result-card success">
      <strong>{t("requestSent")}</strong><br />
      <span class="muted">{t("providerLabel")} {result.provider || provider}</span>
      {#if snapshotId}
        <div class="snapshot-id">
          <span class="snap-label">{t("snapshotId")}</span>
          <span class="snap-value">{snapshotId}</span>
        </div>
      {/if}
    </div>
    {#if resultRaw}
      <pre class="raw-response">{resultRaw}</pre>
    {/if}
  {/if}

  {#if collBusy}
    <div class="spinner">⏳ {t("contacting")}</div>
  {/if}

  <GuideCard title={cloudEvidenceGuide.title} icon={cloudEvidenceGuide.icon} steps={cloudEvidenceGuide.steps} references={cloudEvidenceGuide.references} />
</div>

<style>
  h3 { margin:0 0 8px; font-size:16px; }
  .row { margin-bottom:10px; }
  .cred-row { display:flex; align-items:flex-end; gap:8px; flex-wrap:wrap; }
  label { font-size:13px; display:flex; align-items:center; gap:6px; }
  input, select {
    background: var(--input-bg); color: var(--text); border:1px solid var(--border);
    border-radius:6px; padding:6px 10px; width:320px; font-size:13px;
  }
  input:disabled, select:disabled { opacity: 0.5; }
  .btn-sm {
    padding: 6px 12px; background: var(--btn-secondary-bg); color: var(--btn-secondary-text);
    border: 1px solid var(--border); border-radius: 6px; cursor: pointer; font-size: 12px;
  }
  .btn-primary {
    padding:10px 24px; background:var(--primary); color:white;
    border:none; border-radius:8px; cursor:pointer; font-weight:600; margin-top:12px;
    transition: filter 0.15s;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .note { font-size:11px; color:var(--text-secondary); margin: 0 0 16px; }
  .hint { font-size:10px; color:var(--text-muted); display:block; margin-top:2px; margin-left:2px; }
  .result-card {
    margin-top:12px; padding:10px 14px; border-radius:8px; font-size:13px;
    background: rgba(34,197,94,0.1); border: 1px solid var(--success);
  }
  .result-card.error {
    background: rgba(239,68,68,0.1); border: 1px solid var(--danger); color: var(--danger);
  }
  .muted { color: var(--text-muted); font-size:11px; }
  .snapshot-id { display:flex; gap:6px; margin-top:6px; font-size:12px; align-items:center; }
  .snap-label { color:var(--text-muted); }
  .snap-value { font-family:var(--mono); color:var(--primary); font-weight:600; }
  .raw-response {
    margin-top:12px; padding:10px; background: var(--code-bg); border:1px solid var(--border);
    border-radius:6px; font-size:11px; font-family: var(--mono); max-height:300px;
    overflow:auto; white-space:pre-wrap; word-break:break-all; color: var(--text-secondary);
  }
  .spinner { margin-top:12px; font-size:13px; color: var(--primary); }
</style>
