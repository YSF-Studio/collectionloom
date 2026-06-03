<script>
  import { invoke } from "@tauri-apps/api/core";

  let { filePath, busy, msg, timeoutPromise, onPreview } = $props();

  let preview = $state(null);
  let loading = $state(false);

  // Hex viewer state
  let hexLines = $state([]);
  let selectedOffset = $state(null);
  let selectedBytes = $state("");
  let hexSearch = $state("");
  let hexSearchHits = $state([]);

  export async function loadPreview(path) {
    if (!path) return;
    loading = true;
    hexLines = [];
    selectedOffset = null;
    hexSearch = "";
    hexSearchHits = [];
    try {
      const result = await timeoutPromise(invoke("preview_file", { path }), 30000);
      preview = result;
      if (result.preview?.HexDump) {
        parseHexDump(result.preview.HexDump);
      }
      if (onPreview) onPreview(result);
    } catch (e) {
      preview = { kind: "Error", preview: { Unsupported: String(e) } };
    }
    loading = false;
  }

  export function clear() { preview = null; hexLines = []; }

  $effect(() => { if (filePath) loadPreview(filePath); });

  function parseHexDump(raw) {
    const lines = [];
    for (const line of raw.split("\n")) {
      const m = line.match(/^([0-9a-fA-F]{8})  (.{47}) \|(.{1,16})\|$/);
      if (m) {
        const byteCells = [];
        const hexPart = m[2];
        for (let i = 0; i < hexPart.length; i += 3) {
          const b = hexPart.substring(i, i + 2).trim();
          byteCells.push(b || "");
        }
        lines.push({ offset: parseInt(m[1], 16), hex: byteCells, ascii: m[3] });
      }
    }
    hexLines = lines;
  }

  function onHexClick(lineIdx, byteIdx) {
    if (!hexLines[lineIdx]) return;
    const line = hexLines[lineIdx];
    selectedOffset = line.offset + byteIdx;
    selectedBytes = line.hex[byteIdx] || "";
  }

  function doHexSearch() {
    if (!hexSearch.trim()) { hexSearchHits = []; return; }
    const query = hexSearch.toLowerCase();
    const hits = [];
    for (let li = 0; li < hexLines.length; li++) {
      const line = hexLines[li];
      for (let bi = 0; bi < line.hex.length; bi++) {
        if (line.hex[bi]?.toLowerCase() === query) hits.push({ li, bi });
      }
    }
    hexSearchHits = hits;
  }

  function isSearchHit(li, bi) {
    return hexSearchHits.some(h => h.li === li && h.bi === bi);
  }

  function copyOffset() {
    if (selectedOffset == null) return;
    const hex = `0x${selectedOffset.toString(16).padStart(8, "0")}`;
    navigator.clipboard?.writeText(hex);
    msg?.set && msg.set(`📋 Copied: ${hex}`);
  }

  function sizeStr(bytes) {
    if (!bytes) return "0 B";
    const u = ["B", "KB", "MB", "GB"];
    let i = 0, s = bytes;
    while (s >= 1024 && i < u.length - 1) { s /= 1024; i++; }
    return `${s.toFixed(i === 0 ? 0 : 1)} ${u[i]}`;
  }
</script>

{#if loading}
  <div class="loading"><span class="spinner">⏳</span> Loading preview...</div>
{:else if preview}
  <div class="preview">
    {#if preview.preview?.Text}
      <pre class="text-view">{preview.preview.Text}</pre>
    {:else if preview.preview?.Image}
      <div class="image-view">
        <img src="data:image/png;base64,{preview.preview.Image.data_base64}" alt="preview" />
        <span class="dim">{preview.preview.Image.width} × {preview.preview.Image.height}px</span>
      </div>
    {:else if preview.preview?.HexDump}
      <!-- Hex Toolbar -->
      <div class="hex-toolbar">
        <div class="hex-search">
          <input type="text" bind:value={hexSearch} placeholder="Search byte (e.g. FF)"
            onkeydown={(e) => e.key === "Enter" && doHexSearch()} />
          <button class="btn-ghost" onclick={doHexSearch}>🔍</button>
          {#if hexSearchHits.length > 0}
            <span class="hit-count">{hexSearchHits.length} match{hexSearchHits.length > 1 ? 'es' : ''}</span>
          {/if}
        </div>
        <div class="hex-info">
          {#if selectedOffset != null}
            <span class="sel-info">
              Offset: <code>0x{selectedOffset.toString(16).padStart(8, "0").toUpperCase()}</code>
              {#if selectedBytes}
                | Byte: <code>{selectedBytes.toUpperCase()}</code>
                | Dec: <code>{parseInt(selectedBytes, 16) || "—"}</code>
              {/if}
            </span>
            <button class="btn-ghost btn-copy" onclick={copyOffset}>📋 Copy Offset</button>
          {/if}
        </div>
      </div>

      <!-- Hex Grid -->
      <div class="hex-grid-wrap">
        <div class="hex-grid">
          {#each hexLines as line, li}
            <div class="hex-line">
              <span class="hex-offset">{line.offset.toString(16).padStart(8, "0").toUpperCase()}</span>
              <span class="hex-bytes">
                {#each line.hex as byte, bi}
                  {#if byte}
                    <span class="hex-byte" class:hl={isSearchHit(li, bi)} class:sel={selectedOffset === line.offset + bi}
                      onclick={() => onHexClick(li, bi)}>{byte.toUpperCase()}</span>
                  {:else}
                    <span class="hex-byte empty">  </span>
                  {/if}
                {/each}
              </span>
              <span class="hex-ascii">|{line.ascii}|</span>
            </div>
          {/each}
        </div>
      </div>
    {:else if preview.preview?.ArchiveList}
      <div class="archive-view">
        <h4>📦 Archive Contents ({preview.preview.ArchiveList.length} items)</h4>
        <div class="arc-list">
          {#each preview.preview.ArchiveList as entry}
            <div class="arc-item">{entry}</div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="unsupported"><p>⚠️ No preview available</p></div>
    {/if}

    <div class="file-meta">
      <span class="label">Size:</span> {sizeStr(preview.size)}
      <span class="sep">|</span>
      <span class="label">Type:</span> {preview.kind}
      <span class="sep">|</span>
      <span class="label">MIME:</span> {preview.mime_type}
      <span class="sep">|</span>
      <span class="label">Ext:</span> {preview.extension}
    </div>
  </div>
{:else}
  <div class="empty"><p>Select a file to preview</p></div>
{/if}

<style>
  .loading { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); gap: 8px; font-size: 14px; }
  .spinner { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .preview { display: flex; flex-direction: column; height: 100%; }
  .text-view { flex: 1; overflow: auto; background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; padding: 12px; font-family: "SF Mono","Menlo","Cascadia Code",monospace; font-size: 12px; line-height: 1.5; color: #d4d4d4; white-space: pre-wrap; word-break: break-all; }

  /* ─── Hex Toolbar ─── */
  .hex-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 6px 8px; background: #111; border: 1px solid var(--border); border-radius: 6px 6px 0 0; gap: 8px; flex-shrink: 0; }
  .hex-search { display: flex; align-items: center; gap: 4px; }
  .hex-search input { background: #1a1a1a; border: 1px solid var(--border); border-radius: 4px; color: #ccc; font-size: 11px; padding: 3px 8px; width: 120px; font-family: monospace; }
  .hex-info { display: flex; align-items: center; gap: 8px; font-size: 11px; color: var(--text-secondary); }
  .sel-info code { color: var(--primary); font-family: monospace; }
  .btn-ghost { background: transparent; border: 1px solid var(--border); border-radius: 4px; color: var(--text-secondary); cursor: pointer; font-size: 11px; padding: 2px 8px; }
  .btn-ghost:hover { border-color: var(--primary); color: var(--primary); }
  .btn-copy { font-size: 10px; padding: 2px 6px; }
  .hit-count { color: var(--success); font-size: 10px; margin-left: 4px; }

  /* ─── Hex Grid ─── */
  .hex-grid-wrap { flex: 1; overflow: auto; background: #0d0d0d; border: 1px solid var(--border); border-top: none; border-radius: 0 0 6px 6px; }
  .hex-grid { padding: 4px 0; font-family: "SF Mono","Menlo","Cascadia Code",monospace; font-size: 11px; line-height: 18px; }
  .hex-line { display: flex; align-items: center; padding: 0 8px; white-space: nowrap; }
  .hex-line:hover { background: rgba(59,130,246,0.04); }
  .hex-offset { color: #555; margin-right: 12px; flex-shrink: 0; min-width: 64px; }
  .hex-bytes { display: flex; gap: 0; flex-shrink: 0; }
  .hex-byte { display: inline-block; width: 18px; text-align: center; cursor: pointer; border-radius: 2px; color: #a0c4ff; transition: all 0.1s; }
  .hex-byte:hover { background: rgba(59,130,246,0.2); color: #fff; }
  .hex-byte.sel { background: var(--primary); color: #fff; font-weight: 600; }
  .hex-byte.hl { background: rgba(245,158,11,0.3); color: #fbbf24; }
  .hex-byte.empty { color: transparent; cursor: default; }
  .hex-ascii { color: #888; margin-left: 16px; flex-shrink: 0; }

  .image-view { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 16px; }
  .image-view img { max-width: 100%; border-radius: 6px; border: 1px solid var(--border); }
  .dim { font-size: 11px; color: var(--text-secondary); }
  .archive-view { flex: 1; overflow: auto; }
  .archive-view h4 { margin: 0 0 10px; font-size: 13px; }
  .arc-list { display: flex; flex-direction: column; gap: 2px; }
  .arc-item { padding: 4px 8px; font-family: "SF Mono","Menlo",monospace; font-size: 11px; color: var(--text-secondary); border-bottom: 1px solid var(--border); }
  .unsupported { display: flex; align-items: center; justify-content: center; height: 150px; color: var(--warn); }
  .file-meta { padding: 8px 12px; font-size: 11px; color: var(--text-secondary); background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; margin-top: 8px; flex-shrink: 0; }
  .sep { margin: 0 6px; opacity: 0.3; }
  .label { color: #888; }
  .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); font-size: 14px; }
</style>
