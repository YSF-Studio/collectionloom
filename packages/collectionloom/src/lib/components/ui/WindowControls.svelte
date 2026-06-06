<script>
import { isTauri } from "../../api/tauri.js";
import { getAppWindow, guessPlatform } from "../../window.js";

/** @type {{ variant?: 'macos' | 'windows' | 'linux' | 'auto' }} */
let { variant = "auto" } = $props();

let resolved = $derived(variant === "auto" ? guessPlatform() : variant);

/** @param {(w: import('@tauri-apps/api/window').Window) => Promise<void> | void} action */
async function run(action) {
  if (!isTauri()) return;
  const w = await getAppWindow();
  if (!w) return;
  await action(w);
}
</script>

{#if resolved === "macos"}
  <div class="traffic-lights" role="group" aria-label="Window controls">
    <button
      type="button"
      class="tl red"
      aria-label="Close window"
      onclick={() => run((w) => w.close())}
    ></button>
    <button
      type="button"
      class="tl yellow"
      aria-label="Minimize window"
      onclick={() => run((w) => w.minimize())}
    ></button>
    <button
      type="button"
      class="tl green"
      aria-label="Maximize window"
      onclick={() => run((w) => w.toggleMaximize())}
    ></button>
  </div>
{:else}
  <div class="win-controls" role="group" aria-label="Window controls">
    <button
      type="button"
      class="win-btn minimize"
      aria-label="Minimize window"
      onclick={() => run((w) => w.minimize())}
    >
      &#8212;
    </button>
    <button
      type="button"
      class="win-btn maximize"
      aria-label="Maximize window"
      onclick={() => run((w) => w.toggleMaximize())}
    >
      &#9633;
    </button>
    <button
      type="button"
      class="win-btn close"
      aria-label="Close window"
      onclick={() => run((w) => w.close())}
    >
      &#10005;
    </button>
  </div>
{/if}

<style>
  .traffic-lights {
    display: flex;
    gap: 7px;
  }
  .tl {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    padding: 0;
    cursor: default;
    flex-shrink: 0;
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
  .tl:hover {
    filter: brightness(0.92);
  }
  .tl:active {
    filter: brightness(0.85);
  }

  .win-controls {
    display: flex;
    align-items: stretch;
    height: 28px;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }
  .win-btn {
    width: 36px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    line-height: 1;
    cursor: default;
    padding: 0;
  }
  .win-btn:hover {
    background: var(--btn-secondary-bg);
  }
  .win-btn.close:hover {
    background: #e81123;
    color: #fff;
  }
</style>
