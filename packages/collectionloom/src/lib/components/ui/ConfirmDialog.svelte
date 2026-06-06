<script>
  let {
    open = false,
    title = "Confirm",
    message = "",
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    variant = "danger",
    onConfirm = () => {},
    onCancel = () => {},
  } = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="confirm-overlay" onclick={onCancel} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="confirm-dialog" tabindex="-1" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="confirm-title">
      <h3 id="confirm-title" class="confirm-title">{title}</h3>
      <p class="confirm-message">{message}</p>
      <div class="confirm-actions">
        <button type="button" class="btn-cancel" onclick={onCancel}>{cancelLabel}</button>
        <button type="button" class="btn-confirm {variant}" onclick={onConfirm}>{confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    animation: fadeIn 0.15s ease-out;
  }
  .confirm-dialog {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 20px 24px;
    max-width: 400px;
    width: calc(100% - 40px);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .confirm-title {
    margin: 0 0 10px;
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }
  .confirm-message {
    margin: 0 0 18px;
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .confirm-actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }
  .btn-cancel {
    padding: 8px 16px;
    background: var(--btn-secondary-bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--btn-secondary-text);
    cursor: pointer;
    font-size: 12px;
  }
  .btn-confirm {
    padding: 8px 16px;
    border: none;
    border-radius: 8px;
    color: white;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }
  .btn-confirm.danger {
    background: var(--danger);
  }
  .btn-confirm.primary {
    background: var(--primary);
  }
</style>
