<script>
  /**
   * GuideCard — Collapsible help panel with step-by-step forensic instructions.
   *
   * Props:
   *   title       (string)       — Heading text
   *   icon        (string)       — Emoji or icon displayed above the title
   *   steps       (array of { title, description, warning? })
   *                              — Ordered procedure steps
   *   references  (string[], optional)
   *                              — Citation / resource links
   */
  let { title = "", icon = "", steps = [], references = [] } = $props();

  let expanded = $state(false);

  function toggle() {
    expanded = !expanded;
  }
</script>

<div class="guide-card" class:expanded>
  <button class="guide-header" onclick={toggle} aria-expanded={expanded}>
    <span class="guide-icon">{icon}</span>
    <span class="guide-title">{title}</span>
    <span class="chevron">{expanded ? "▾" : "▸"}</span>
  </button>

  {#if expanded}
    <div class="guide-body">
      <!-- Steps list -->
      <ol class="steps-list">
        {#each steps as step, i}
          <li class="step-item">
            <div class="step-marker">{i + 1}</div>
            <div class="step-content">
              <strong class="step-title">{step.title}</strong>
              <p class="step-desc">{step.description}</p>
              {#if step.warning}
                <div class="step-warning">
                  <span class="warn-icon">Note:</span>
                  <span>{step.warning}</span>
                </div>
              {/if}
            </div>
          </li>
        {/each}
      </ol>

      <!-- References section -->
      {#if references.length > 0}
        <div class="references">
          <div class="ref-header">References</div>
          <ul class="ref-list">
            {#each references as ref}
              <li class="ref-item">{ref}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .guide-card {
    background: var(--card, #141414);
    border: 1px solid var(--border, #2a2a2a);
    border-radius: var(--radius-lg, 12px);
    margin-bottom: 12px;
    overflow: hidden;
    transition: border-color 0.15s;
  }
  .guide-card.expanded {
    border-color: var(--border-light, #333);
  }

  .guide-header {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    color: var(--text, #e0e0e0);
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    font-family: inherit;
    text-align: left;
    transition: background 0.15s;
    -webkit-app-region: no-drag;
  }
  .guide-header:hover {
    background: var(--card-hover, #1a1a1a);
  }

  .guide-icon {
    font-size: 18px;
    line-height: 1;
    flex-shrink: 0;
  }

  .guide-title {
    flex: 1;
  }

  .chevron {
    font-size: 12px;
    color: var(--text-secondary, #86868b);
    flex-shrink: 0;
    transition: transform 0.15s;
  }

  .guide-body {
    padding: 0 16px 16px;
    animation: fadeIn 0.2s ease-out;
  }

  /* Steps */
  .steps-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .step-item {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .step-marker {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--primary-bg, rgba(59,130,246,0.12));
    color: var(--primary, #3b82f6);
    font-size: 11px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    line-height: 1;
  }

  .step-content {
    flex: 1;
    min-width: 0;
  }

  .step-title {
    display: block;
    font-size: 13px;
    color: var(--text, #e0e0e0);
    margin-bottom: 2px;
  }

  .step-desc {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary, #86868b);
    line-height: 1.5;
  }

  .step-warning {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    margin-top: 6px;
    padding: 6px 10px;
    background: var(--warn-bg, rgba(245,158,11,0.15));
    border: 1px solid rgba(245,158,11,0.3);
    border-radius: var(--radius, 8px);
    font-size: 11px;
    color: var(--warn, #f59e0b);
    line-height: 1.4;
  }

  .warn-icon {
    flex-shrink: 0;
    font-size: 13px;
  }

  /* References */
  .references {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--border, #2a2a2a);
  }

  .ref-header {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, #86868b);
    margin-bottom: 8px;
  }

  .ref-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .ref-item {
    font-size: 11px;
    color: var(--text-muted, #555);
    padding: 4px 8px;
    background: rgba(255,255,255,0.03);
    border-radius: var(--radius-sm, 4px);
    line-height: 1.4;
  }
</style>
