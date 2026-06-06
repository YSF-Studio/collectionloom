<script>
  let { format = $bindable("raw"), hashMd5 = $bindable(true), hashSha256 = $bindable(true), disabled = false } = $props();
</script>

<div class="format-picker">
  <div class="format-row">
    <span class="label">Format:</span>
    <label class="radio-opt"><input type="radio" bind:group={format} value="e01" {disabled} /> E01</label>
    <label class="radio-opt"><input type="radio" bind:group={format} value="raw" {disabled} /> RAW/DD</label>
    <label class="radio-opt"><input type="radio" bind:group={format} value="aff4" {disabled} /> AFF4</label>
  </div>
  <div class="format-row">
    <span class="label">Hashing:</span>
    <label class="check-opt"><input type="checkbox" bind:checked={hashMd5} {disabled} /> MD5</label>
    <label class="check-opt"><input type="checkbox" bind:checked={hashSha256} {disabled} /> SHA-256</label>
  </div>
  {#if format === "aff4"}
    <p class="hint">AFF4 support is planned — V1 uses RAW/DD with sidecar metadata.</p>
  {/if}
</div>

<style>
  .format-picker { display: flex; flex-direction: column; gap: 10px; }
  .format-row { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; font-size: 13px; }
  .label { color: var(--text-secondary, #86868b); min-width: 56px; }
  .radio-opt, .check-opt { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text, #e0e0e0); }
  .hint { margin: 0; font-size: 11px; color: var(--warn, #f59e0b); }
</style>
