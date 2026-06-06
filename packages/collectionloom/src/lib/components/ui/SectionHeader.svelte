<script>
  let { title = "", subtitle = "" } = $props();
</script>

<div class="section-header">
  <h2 class="title">{title}</h2>
  {#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
</div>

<style>
  .section-header { margin-bottom: 20px; }
  .title { margin: 0; font-size: 20px; font-weight: 600; color: var(--text, #e0e0e0); letter-spacing: -0.02em; }
  .subtitle { margin: 4px 0 0; font-size: 13px; color: var(--text-secondary, #86868b); }
</style>
