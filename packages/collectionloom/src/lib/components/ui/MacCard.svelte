<script>
  let { title = "", subtitle = "", class: className = "", children } = $props();
</script>

<div class="mac-card {className}">
  {#if title}
    <div class="card-header">
      <h4 class="card-title">{title}</h4>
      {#if subtitle}<p class="card-subtitle">{subtitle}</p>{/if}
    </div>
  {/if}
  <div class="card-body">
    {@render children()}
  </div>
</div>

<style>
  .mac-card {
    background: var(--card, #141414);
    border: 1px solid var(--border, #2a2a2a);
    border-radius: var(--radius-lg, 12px);
    padding: 16px 18px;
    margin-bottom: 14px;
  }
  .card-header { margin-bottom: 12px; }
  .card-title { margin: 0; font-size: 13px; font-weight: 600; color: var(--text, #e0e0e0); }
  .card-subtitle { margin: 4px 0 0; font-size: 11px; color: var(--text-secondary, #86868b); }
  .card-body { display: flex; flex-direction: column; gap: 10px; }
</style>
