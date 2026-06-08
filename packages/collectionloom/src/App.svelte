<script>
import DiskTab from "./lib/components/DiskTab.svelte";
import RamTab from "./lib/components/RamTab.svelte";
import MobileTab from "./lib/components/MobileTab.svelte";
import CloudTab from "./lib/components/CloudTab.svelte";
import NetworkTab from "./lib/components/NetworkTab.svelte";
import EncryptionTab from "./lib/components/EncryptionTab.svelte";
import VerificationTab from "./lib/components/VerificationTab.svelte";
import CocTab from "./lib/components/CocTab.svelte";
import SnapshotTab from "./lib/components/SnapshotTab.svelte";
import ExportTab from "./lib/components/ExportTab.svelte";
import DashboardTab from "./lib/components/DashboardTab.svelte";
import DisclaimerTab from "./lib/components/DisclaimerTab.svelte";
import AcquireAllTab from "./lib/components/AcquireAllTab.svelte";
import PreflightTab from "./lib/components/PreflightTab.svelte";
import PillBadge from "./lib/components/ui/PillBadge.svelte";
import ProgressStatusBar from "./lib/components/ui/ProgressStatusBar.svelte";
import LocaleToggle from "./lib/components/ui/LocaleToggle.svelte";
import ThemeToggle from "./lib/components/ui/ThemeToggle.svelte";
import ConfirmDialog from "./lib/components/ui/ConfirmDialog.svelte";
import { guessPlatform } from "./lib/window.js";
import { invoke, isTauri } from "./lib/api/tauri.js";
import { isError, isWarn } from "./lib/messages.js";
import { initLocale, subscribeLocale } from "./lib/stores/locale.js";

let activeSection = $state("disk");
let msg = $state("");
let busy = $state(false);
let wbActive = $state(false);
let wbBusy = $state(false);

let diskState = {};
let ramState = {};
let encryptionState = {};
let cocState = { caseId: "", operator: "" };
let wbDevice = $state("");
let wbDisks = $state([]);
let wbDisksLoading = $state(false);
let showConfirmDisableWb = $state(false);
let acquireAllState = {};
let locale = $state("en");
let platform = $state(guessPlatform());

let statusBar = $state({
  active: false,
  percent: 0,
  label: "",
  eta: "",
});

function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

$effect(() => {
  initLocale();
  const unsubscribe = subscribeLocale((_, resolved) => {
    locale = resolved;
  });
  return unsubscribe;
});

function setBusy(v) {
  busy = v;
}
function setMsg(m) {
  msg = m;
}

const dictionary = {
  en: {
    acquisition: "ACQUISITION",
    analysis: "ANALYSIS",
    caseInfo: "CASE INFO",
    writeBlockerActive: "Write-Blocker Active",
    writeBlockerInactive: "Write-Blocker Inactive",
    selectWbDisk: "Select disk for write-blocker",
    refreshDiskList: "Refresh disk list",
    selectDisk: "— Select disk —",
    warnMissingPrereq: "WARN: {count} prerequisite tool(s) missing — open Prerequisites tab",
    selectTargetFirst: "WARN: Select a target disk in the titlebar first",
  },
  id: {
    acquisition: "AKUISISI",
    analysis: "ANALISIS",
    caseInfo: "INFO KASUS",
    writeBlockerActive: "Write-Blocker Aktif",
    writeBlockerInactive: "Write-Blocker Nonaktif",
    selectWbDisk: "Pilih disk untuk write-blocker",
    refreshDiskList: "Segarkan daftar disk",
    selectDisk: "— Pilih disk —",
    warnMissingPrereq: "PERINGATAN: {count} tool prasyarat hilang — buka tab Prerequisites",
    selectTargetFirst: "PERINGATAN: Pilih disk target di titlebar dulu",
  },
};

function t(key, vars = {}) {
  const text = dictionary[locale]?.[key] || dictionary.en[key] || key;
  return text.replace(/\{(\w+)\}/g, (_, name) => String(vars[name] ?? ""));
}

function handleDiskProgress({ progress, collBusy, eta, selectedDisk, imageFormat }) {
  if (collBusy && progress) {
    statusBar = {
      active: true,
      percent: progress.percent || 0,
      label: `Imaging ${selectedDisk || "drive"} (${(imageFormat || "raw").toUpperCase()})…`,
      eta: eta || "",
    };
  } else if (!collBusy) {
    statusBar = { active: false, percent: 0, label: "", eta: "" };
  }
}

function handleAcquireProgress({ label, percent, busy: isBusy }) {
  if (isBusy && label) {
    statusBar = { active: true, percent: percent || 0, label, eta: "" };
  } else if (!isBusy) {
    statusBar = { active: false, percent: 0, label: "", eta: "" };
  }
}

function handleDeviceSelect({ device, wbActive: active }) {
  wbDevice = device;
  wbActive = active;
}

async function loadWbDisks() {
  wbDisksLoading = true;
  try {
    wbDisks = await timeoutPromise(invoke("list_disks"), 15000);
  } catch {
    wbDisks = [];
  }
  wbDisksLoading = false;
}

async function refreshWbStatus() {
  if (!wbDevice) {
    wbActive = false;
    return;
  }
  try {
    const r = await invoke("check_write_blocker", { device: wbDevice });
    const active = !!(r?.active ?? r?.enabled);
    wbActive = active;
  } catch {
    wbActive = false;
  }
}

async function onWbDeviceChange() {
  await refreshWbStatus();
}

let wbTitle = $derived(
  wbDevice
    ? wbActive
      ? `Disable write-blocker on ${wbDevice}`
      : `Enable write-blocker on ${wbDevice}`
    : "Select a disk, then enable write-blocker"
);

let wbSelectedDiskLabel = $derived.by(() => {
  if (!wbDevice) return "Select disk for write-blocker";
  const disk = wbDisks.find((d) => d.device === wbDevice);
  if (!disk) return wbDevice;
  return `${disk.device} · ${disk.model || "Unknown"} (${(disk.sizeBytes / 1e9).toFixed(1)} GB)`;
});

function requestToggleWriteBlocker() {
  if (!wbDevice || wbBusy || busy) {
    if (!wbDevice) setMsg(t("selectTargetFirst"));
    return;
  }
  if (wbActive) {
    showConfirmDisableWb = true;
    return;
  }
  toggleWriteBlocker();
}

async function toggleWriteBlocker() {
  showConfirmDisableWb = false;
  if (!wbDevice || wbBusy || busy) return;
  wbBusy = true;
  const enabling = !wbActive;
  try {
    const cmd = enabling ? "enable_write_blocker" : "disable_write_blocker";
    const r = await timeoutPromise(invoke(cmd, { device: wbDevice }), 15000);
    const active = !!(r?.active ?? r?.enabled);
    wbActive = active;
    setMsg(
      enabling
        ? active
          ? "OK: Software write-blocker enabled"
          : "WARN: Write-blocker not confirmed"
        : "WARN: Software write-blocker disabled — writes may now be possible"
    );
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  wbBusy = false;
}

const sidebarSections = [
  {
    labelKey: "acquisition",
    items: [
      { id: "disk", labelKey: "disk" },
      { id: "ram", labelKey: "ram" },
      { id: "mobile", labelKey: "mobile" },
      { id: "cloud", labelKey: "cloud" },
      { id: "network", labelKey: "network" },
      { id: "snapshot", labelKey: "snapshot" },
      { id: "acquire-all", labelKey: "acquireAll" },
    ],
  },
  {
    labelKey: "analysis",
    items: [
      { id: "encryption", labelKey: "encryption" },
      { id: "verify", labelKey: "hashVerify" },
    ],
  },
  {
    labelKey: "caseInfo",
    items: [
      { id: "dashboard", labelKey: "caseDashboard" },
      { id: "preflight", labelKey: "prerequisites" },
      { id: "coc", labelKey: "custodyChain" },
      { id: "export", labelKey: "exportBundle" },
      { id: "about", labelKey: "about" },
    ],
  },
];

const navLabels = {
  disk: { en: "Disk Imaging", id: "Imaging Disk" },
  ram: { en: "RAM Capture", id: "Tangkap RAM" },
  mobile: { en: "Mobile Triage", id: "Triage Mobile" },
  cloud: { en: "Cloud Snapshot", id: "Snapshot Cloud" },
  network: { en: "Network Capture", id: "Tangkap Jaringan" },
  snapshot: { en: "System Snapshot", id: "Snapshot Sistem" },
  acquireAll: { en: "Acquire All", id: "Akuisisi Semua" },
  encryption: { en: "Encryption", id: "Enkripsi" },
  hashVerify: { en: "Hash Verify", id: "Verifikasi Hash" },
  caseDashboard: { en: "Case Dashboard", id: "Dasbor Kasus" },
  prerequisites: { en: "Prerequisites", id: "Prasyarat" },
  custodyChain: { en: "Custody Chain", id: "Rantai Custody" },
  exportBundle: { en: "Export Bundle", id: "Paket Ekspor" },
  about: { en: "About", id: "Tentang" },
};

function navLabel(key) {
  return navLabels[key]?.[locale] || navLabels[key]?.en || key;
}

  $effect(() => {
  loadWbDisks();
  if (isTauri()) {
    invoke("run_preflight_check")
      .then((r) => {
        if (r?.missingCount > 0) {
          setMsg(t("warnMissingPrereq", { count: r.missingCount }));
        } else if (r?.warningCount > 0) {
          setMsg(`WARN: ${r.summary}`);
        }
      })
      .catch(() => {});
  }
  const applyHash = () => {
    const id = window.location.hash.replace(/^#/, "");
    if (id && window.__sections?.includes(id)) activeSection = id;
  };
  applyHash();
  window.addEventListener("hashchange", applyHash);
  return () => window.removeEventListener("hashchange", applyHash);
});

window.__goTo = (id) => {
  activeSection = id;
  window.location.hash = id;
};
window.__sections = sidebarSections.flatMap((s) => s.items.map((i) => i.id));
</script>

<div class="app-shell">
  <div class="titlebar">
    <div class="titlebar-brand">
      <img src="/icon.png" class="logo" alt="CollectionLoom" />
      <span class="title">CollectionLoom</span>
      {#if wbActive}
        <PillBadge variant="active" label={t("writeBlockerActive")} />
      {:else}
        <PillBadge variant="inactive" label={t("writeBlockerInactive")} />
      {/if}
    </div>
    <div class="wb-titlebar-controls">
        <select
          class="wb-device-select"
          bind:value={wbDevice}
          onchange={onWbDeviceChange}
          disabled={wbBusy || wbDisksLoading}
          title={wbSelectedDiskLabel}
          aria-label={t("selectWbDisk")}
        >
          <option value="">{t("selectDisk")}</option>
          {#each wbDisks as disk}
            <option value={disk.device}>
              {disk.device} · {disk.model || "Unknown"} ({(disk.sizeBytes / 1e9).toFixed(1)} GB)
            </option>
          {/each}
        </select>
        <button
          type="button"
          class="wb-icon-btn"
          onclick={loadWbDisks}
          disabled={wbDisksLoading}
          title={t("refreshDiskList")}
          aria-label={t("refreshDiskList")}
        >
          {wbDisksLoading ? "…" : "↻"}
        </button>
        <button
          type="button"
          class="wb-titlebar-btn"
          class:wb-on={wbActive}
          onclick={requestToggleWriteBlocker}
          disabled={wbBusy || busy || !wbDevice}
          title={wbTitle}
          aria-label={wbActive ? "Disable software write-blocker" : "Enable software write-blocker"}
        >
          {#if wbBusy}
            …
          {:else if wbActive}
            Disable WB
          {:else}
            Enable WB
          {/if}
        </button>
    </div>
    <div class="titlebar-end">
      <LocaleToggle />
      <ThemeToggle />
    </div>
  </div>

  <div class="two-pane">
    <aside class="sidebar">
      {#each sidebarSections as section}
        <div class="sidebar-group">
          <span class="sidebar-label">{t(section.labelKey)}</span>
          {#each section.items as item}
            <button
              class="sidebar-item"
              class:active={activeSection === item.id}
              data-nav-id={item.id}
              onclick={() => (activeSection = item.id)}
            >
              {navLabel(item.labelKey)}
            </button>
          {/each}
        </div>
      {/each}
    </aside>

    <div class="content-area">
      {#if activeSection === "disk"}
        <DiskTab
          busy={busy}
          sharedState={diskState}
          caseState={cocState}
          wbDevice={wbDevice}
          {setBusy}
          {setMsg}
          {timeoutPromise}
          onProgressChange={handleDiskProgress}
          onDeviceSelect={handleDeviceSelect}
        />
      {:else if activeSection === "ram"}
        <RamTab busy={busy} sharedState={ramState} caseState={cocState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "mobile"}
        <MobileTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "cloud"}
        <CloudTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "network"}
        <NetworkTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "snapshot"}
        <SnapshotTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "acquire-all"}
        <AcquireAllTab
          sharedState={acquireAllState}
          caseState={cocState}
          busy={busy}
          {setBusy}
          {setMsg}
          {timeoutPromise}
          onProgressChange={handleAcquireProgress}
          onDeviceSelect={handleDeviceSelect}
        />
      {:else if activeSection === "encryption"}
        <EncryptionTab busy={busy} sharedState={encryptionState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "verify"}
        <VerificationTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "coc"}
        <CocTab busy={busy} sharedState={cocState} caseState={cocState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "dashboard"}
        <DashboardTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "preflight"}
        <PreflightTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "export"}
        <ExportTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "about"}
        <DisclaimerTab />
      {/if}
    </div>
  </div>

  <div class="platform-hint">
    <span class="platform-pill">{platform}</span>
  </div>

  <ProgressStatusBar
    active={statusBar.active}
    percent={statusBar.percent}
    label={statusBar.label}
    eta={statusBar.eta}
    wbActive={wbActive}
    busy={busy}
  />

  {#if msg}
    <div class="toast" class:error={isError(msg)} class:warn={isWarn(msg)}>
      {msg}
      <button class="close-toast" onclick={() => (msg = "")} aria-label="Close">×</button>
    </div>
  {/if}

  <ConfirmDialog
    open={showConfirmDisableWb}
    title="Disable Write-Blocker?"
    message="Disabling the software write-blocker can immediately allow writes to the selected source. Only proceed after imaging is complete or when you intentionally need write access."
    confirmLabel="Disable Write-Blocker"
    variant="danger"
    onConfirm={toggleWriteBlocker}
    onCancel={() => (showConfirmDisableWb = false)}
  />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    height: 100vh;
    background: var(--bg);
    color: var(--text);
    font-family: var(--font);
    transition: background 0.2s, color 0.2s;
  }
  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: border-box;
  }

  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg);
  }

  .titlebar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 14px;
    background: var(--shell-titlebar);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--shell-border);
    transition: background 0.2s, border-color 0.2s;
  }
  .titlebar-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .wb-titlebar-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
    justify-content: flex-end;
  }
  .wb-device-select {
    flex: 1 1 auto;
    min-width: 140px;
    max-width: 360px;
    padding: 4px 8px;
    font-size: 11px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--input-bg);
    color: var(--text);
    cursor: pointer;
  }
  .wb-icon-btn {
    padding: 4px 7px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--btn-secondary-bg);
    color: var(--btn-secondary-text);
    cursor: pointer;
    line-height: 1;
  }
  .wb-icon-btn:disabled,
  .wb-device-select:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .titlebar-end {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    flex-shrink: 0;
  }
  .logo {
    width: 18px;
    height: 18px;
    border-radius: 3px;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
    color: var(--title-text);
  }
  .wb-titlebar-btn {
    -webkit-app-region: no-drag;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 600;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--btn-secondary-bg);
    color: var(--btn-secondary-text);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }
  .wb-titlebar-btn:hover:not(:disabled) {
    background: var(--sidebar-hover);
    border-color: var(--primary);
    color: var(--primary);
  }
  .wb-titlebar-btn.wb-on {
    background: var(--success-bg);
    border-color: var(--success);
    color: var(--success);
  }
  .wb-titlebar-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .two-pane {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 210px;
    min-width: 210px;
    background: var(--shell-sidebar);
    backdrop-filter: blur(20px);
    border-right: 1px solid var(--shell-border);
    overflow-y: auto;
    padding: 8px 0;
    transition: background 0.2s, border-color 0.2s;
  }
  .sidebar-group {
    margin-bottom: 4px;
  }
  .sidebar-label {
    display: block;
    padding: 6px 14px 3px;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: calc(100% - 16px);
    padding: 6px 14px;
    margin: 0 8px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    transition: all 0.12s;
  }
  .sidebar-item:hover {
    background: var(--sidebar-hover);
    color: var(--text);
  }
  .sidebar-item.active {
    background: var(--primary-bg);
    color: var(--primary);
    font-weight: 600;
  }
  .content-area {
    flex: 1;
    overflow-y: auto;
    padding: 28px 32px;
    background: var(--bg);
  }

  .platform-hint {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 32px 0;
    color: var(--text-muted);
    font-size: 11px;
  }
  .platform-pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--card);
    border: 1px solid var(--border);
    text-transform: uppercase;
    letter-spacing: 0.6px;
    font-weight: 700;
  }
  .toast {
    position: fixed;
    top: 56px;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 16px;
    border-radius: 10px;
    background: var(--success-bg);
    border: 1px solid var(--success);
    color: var(--text);
    font-size: 12px;
    max-width: min(90vw, 480px);
    z-index: 1000;
    animation: slideUp 0.25s ease-out;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  }
  .toast.error {
    background: var(--danger-bg);
    border-color: var(--danger);
  }
  .toast.warn {
    background: var(--warn-bg);
    border-color: var(--warn);
  }
  .close-toast {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    margin-left: 10px;
  }
</style>
