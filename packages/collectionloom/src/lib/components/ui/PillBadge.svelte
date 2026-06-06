<script>
  let { variant = "inactive", label = "" } = $props();
</script>

<span class="pill-badge {variant}">{label}</span>

<style>
  .pill-badge {
    padding: 2px 10px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 600;
    white-space: nowrap;
  }
  .pill-badge.active {
    background: rgba(34, 197, 94, 0.12);
    color: #22c55e;
  }
  .pill-badge.inactive {
    background: rgba(239, 68, 68, 0.08);
    color: #ef4444;
  }
  .pill-badge.warning {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
  }
  .pill-badge.info {
    background: rgba(59, 130, 246, 0.12);
    color: #3b82f6;
  }
</style>
