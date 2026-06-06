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
import PillBadge from "./lib/components/ui/PillBadge.svelte";
import ProgressStatusBar from "./lib/components/ui/ProgressStatusBar.svelte";
import ThemeToggle from "./lib/components/ui/ThemeToggle.svelte";
import { invoke, isTauri } from "./lib/api/tauri.js";
import { isError, isWarn } from "./lib/messages.js";

let activeSection = $state("disk");
let msg = $state("");
let busy = $state(false);
let wbActive = $state(false);
let wbBusy = $state(false);

let diskState = {};
let ramState = {};
let encryptionState = {};
let cocState = { caseId: "", operator: "" };
let wbState = $state({ active: false, device: "" });
let wbDisks = $state([]);
let wbDisksLoading = $state(false);
let acquireAllState = {};

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

function setBusy(v) {
  busy = v;
}
function setMsg(m) {
  msg = m;
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
  wbState.device = device;
  wbState.active = active;
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
  if (!wbState.device) {
    wbState.active = false;
    wbActive = false;
    return;
  }
  try {
    const r = await invoke("check_write_blocker", { device: wbState.device });
    const active = !!(r?.active ?? r?.enabled);
    wbState.active = active;
    wbActive = active;
  } catch {
    wbState.active = false;
    wbActive = false;
  }
}

async function onWbDeviceChange() {
  await refreshWbStatus();
}

let wbTitle = $derived(
  wbState.device
    ? wbActive
      ? `Disable write-blocker on ${wbState.device}`
      : `Enable write-blocker on ${wbState.device}`
    : "Select a disk, then enable write-blocker"
);

async function toggleWriteBlocker() {
  if (!wbState.device || wbBusy || busy) {
    if (!wbState.device) setMsg("WARN: Select a target disk in the titlebar first");
    return;
  }
  wbBusy = true;
  const enabling = !wbActive;
  try {
    const cmd = enabling ? "enable_write_blocker" : "disable_write_blocker";
    const r = await timeoutPromise(invoke(cmd, { device: wbState.device }), 15000);
    const active = !!(r?.active ?? r?.enabled);
    wbState.active = active;
    wbActive = active;
    setMsg(
      enabling
        ? active
          ? "OK: Software write-blocker enabled"
          : "WARN: Write-blocker not confirmed"
        : "OK: Write-blocker disabled"
    );
  } catch (e) {
    setMsg(`ERR: ${typeof e === "string" ? e : String(e)}`);
  }
  wbBusy = false;
}

const sidebarSections = [
  {
    label: "ACQUISITION",
    items: [
      { id: "disk", label: "Disk Imaging" },
      { id: "ram", label: "RAM Capture" },
      { id: "mobile", label: "Mobile Triage" },
      { id: "cloud", label: "Cloud Snapshot" },
      { id: "network", label: "Network Capture" },
      { id: "snapshot", label: "System Snapshot" },
      { id: "acquire-all", label: "Acquire All" },
    ],
  },
  {
    label: "ANALYSIS",
    items: [
      { id: "encryption", label: "Encryption" },
      { id: "verify", label: "Hash Verify" },
    ],
  },
  {
    label: "CASE INFO",
    items: [
      { id: "dashboard", label: "Case Dashboard" },
      { id: "coc", label: "Custody Chain" },
      { id: "export", label: "Export Bundle" },
      { id: "about", label: "About" },
    ],
  },
];

$effect(() => {
  loadWbDisks();
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
    <div class="traffic-lights">
      <span class="tl red"></span><span class="tl yellow"></span><span class="tl green"></span>
    </div>
    <div class="titlebar-center">
      <img src="/icon.png" class="logo" alt="CollectionLoom" />
      <span class="title">CollectionLoom</span>
      {#if wbActive}
        <PillBadge variant="active" label="Write-Blocker Active" />
      {:else}
        <PillBadge variant="inactive" label="Write-Blocker Inactive" />
      {/if}
      <div class="wb-titlebar-controls">
        <select
          class="wb-device-select"
          bind:value={wbState.device}
          onchange={onWbDeviceChange}
          disabled={wbBusy || wbDisksLoading}
          title="Target disk for write-blocker"
          aria-label="Select disk for write-blocker"
        >
          <option value="">— Select disk —</option>
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
          title="Refresh disk list"
          aria-label="Refresh disk list"
        >
          {wbDisksLoading ? "…" : "↻"}
        </button>
        <button
          type="button"
          class="wb-titlebar-btn"
          class:wb-on={wbActive}
          onclick={toggleWriteBlocker}
          disabled={wbBusy || busy || !wbState.device}
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
    </div>
    <div class="titlebar-end">
      {#if !isTauri()}
        <PillBadge variant="warning" label="Preview" />
      {/if}
      <ThemeToggle />
    </div>
  </div>

  <div class="two-pane">
    <aside class="sidebar">
      {#each sidebarSections as section}
        <div class="sidebar-group">
          <span class="sidebar-label">{section.label}</span>
          {#each section.items as item}
            <button
              class="sidebar-item"
              class:active={activeSection === item.id}
              data-nav-id={item.id}
              onclick={() => (activeSection = item.id)}
            >
              {item.label}
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
      {:else if activeSection === "export"}
        <ExportTab busy={busy} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "about"}
        <DisclaimerTab />
      {/if}
    </div>
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
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    height: 44px;
    padding: 0 14px;
    gap: 12px;
    background: var(--shell-titlebar);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--shell-border);
    -webkit-app-region: drag;
    transition: background 0.2s, border-color 0.2s;
  }
  .traffic-lights {
    display: flex;
    gap: 7px;
    -webkit-app-region: no-drag;
    grid-column: 1;
  }
  .tl {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }
  .tl.red {
    background: #ff5f57;
  }
  .tl.yellow {
    background: #ffbd2e;
  }
  .tl.green {
    background: #28c840;
  }
  .titlebar-center {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
    -webkit-app-region: drag;
    grid-column: 2;
    min-width: 0;
    flex-wrap: wrap;
  }
  .wb-titlebar-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    -webkit-app-region: no-drag;
    max-width: min(100%, 420px);
  }
  .wb-device-select {
    flex: 1;
    min-width: 0;
    max-width: 220px;
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
    -webkit-app-region: no-drag;
    grid-column: 3;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
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
  .icon {
    font-size: 11px;
    opacity: 0.8;
    width: 14px;
    text-align: center;
  }

  .content-area {
    flex: 1;
    overflow-y: auto;
    padding: 28px 32px;
    background: var(--bg);
  }

  .toast {
    position: fixed;
    bottom: 44px;
    right: 20px;
    padding: 10px 16px;
    border-radius: 10px;
    background: var(--success-bg);
    border: 1px solid var(--success);
    color: var(--text);
    font-size: 12px;
    max-width: 380px;
    z-index: 1000;
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
