<script>
  import { getTheme, cycleTheme } from "../../stores/theme.js";

  let mode = $state(getTheme());

  function onCycle() {
    mode = cycleTheme();
  }
</script>

<button
  class="theme-toggle"
  type="button"
  onclick={onCycle}
  title={`Theme: ${mode} (click to cycle)`}
  aria-label={`Theme: ${mode}. Click to cycle light, dark, system.`}
>
  <span class="track">
    <span class="thumb" class:pos-light={mode === "light"} class:pos-dark={mode === "dark"} class:pos-system={mode === "system"}></span>
    <span class="icon" class:active={mode === "light"} aria-hidden="true">L</span>
    <span class="icon" class:active={mode === "dark"} aria-hidden="true">D</span>
    <span class="icon sys" class:active={mode === "system"} aria-hidden="true">S</span>
  </span>
</button>

<style>
  .theme-toggle {
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    -webkit-app-region: no-drag;
  }
  .track {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    align-items: center;
    width: 72px;
    height: 26px;
    border-radius: 13px;
    border: 1px solid var(--border);
    background: var(--card);
    transition: background 0.2s, border-color 0.2s;
  }
  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
    transition: transform 0.2s ease;
  }
  .thumb.pos-light { transform: translateX(0); }
  .thumb.pos-dark { transform: translateX(23px); }
  .thumb.pos-system { transform: translateX(46px); }
  .icon {
    z-index: 1;
    font-size: 10px;
    text-align: center;
    line-height: 26px;
    color: var(--text-muted);
    transition: color 0.2s;
  }
  .icon.sys { font-size: 9px; }
  .icon.active { color: white; }
  .theme-toggle:hover .track {
    border-color: var(--border-light);
    background: var(--card-hover);
  }
</style>
