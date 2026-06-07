<script>
  import { getResolvedLocale, subscribeLocale } from "../../stores/locale.js";

  let {
    active = false,
    percent = 0,
    label = "",
    eta = "",
    wbActive = false,
    busy = false,
  } = $props();

  let locale = $state(getResolvedLocale());

  const text = {
    en: { processing: "Processing…", offline: "Offline", name: "CollectionLoom — Forensic Acquisition Toolkit" },
    id: { processing: "Memproses…", offline: "Offline", name: "CollectionLoom — Toolkit Akuisisi Forensik" },
  };

  const t = (key) => text[locale]?.[key] ?? text.en[key] ?? key;

  const unsubscribe = subscribeLocale((_, resolved) => {
    locale = resolved;
  });

  $effect(() => () => unsubscribe());
</script>

<div class="statusbar">
  <div class="sb-left">
    <span class="status-dot" class:on={wbActive} class:busy={busy || active}></span>
    {#if active && label}
      <span class="status-text">{percent.toFixed(0)}% • {label}{#if eta} — ETA {eta}{/if}</span>
      <div class="mini-bar"><div class="mini-fill" style="width:{percent}%"></div></div>
    {:else if busy}
      <span class="status-text">{t("processing")}</span>
    {:else}
      <span class="status-text">{t("name")}</span>
    {/if}
  </div>
  <div class="sb-right">
    <span class="offline-badge">{t("offline")}</span>
  </div>
</div>

<style>
  .statusbar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0 14px; height: 26px;
    background: var(--shell-statusbar, rgba(10, 10, 10, 0.95));
    border-top: 1px solid var(--shell-border, rgba(255, 255, 255, 0.06));
    font-size: 11px; color: var(--text-muted);
    transition: background 0.2s, border-color 0.2s;
  }
  .sb-left, .sb-right { display: flex; align-items: center; gap: 8px; flex: 1; }
  .sb-right { flex: 0; justify-content: flex-end; }
  .status-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--text-muted); flex-shrink: 0; }
  .status-dot.on { background: var(--success); box-shadow: 0 0 3px var(--success); }
  .status-dot.busy { background: var(--warn); animation: pulse 1s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }
  .status-text { color: var(--text-secondary, #86868b); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mini-bar { flex: 1; max-width: 200px; height: 4px; background: var(--border); border-radius: 2px; overflow: hidden; }
  .mini-fill { height: 100%; background: var(--primary); transition: width 0.3s; }
  .offline-badge {
    padding: 0 6px; background: var(--success-bg);
    color: var(--success); border-radius: 8px; font-size: 10px; font-weight: 600;
  }
</style>
