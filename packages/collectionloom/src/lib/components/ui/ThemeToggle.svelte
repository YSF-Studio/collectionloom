<script>
  import { getTheme, toggleTheme } from "../../stores/theme.js";

  let mode = $state(getTheme());

  function onToggle() {
    mode = toggleTheme();
  }
</script>

<button
  class="theme-toggle"
  type="button"
  onclick={onToggle}
  title={mode === "dark" ? "Switch to Light Mode" : "Switch to Dark Mode"}
  aria-label={mode === "dark" ? "Switch to Light Mode" : "Switch to Dark Mode"}
>
  <span class="track">
    <span class="thumb" class:light={mode === "light"}></span>
    <span class="icon sun" class:active={mode === "light"} aria-hidden="true">L</span>
    <span class="icon moon" class:active={mode === "dark"} aria-hidden="true">D</span>
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
    grid-template-columns: 1fr 1fr;
    align-items: center;
    width: 52px;
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
  .thumb.light {
    transform: translateX(26px);
  }
  .icon {
    z-index: 1;
    font-size: 11px;
    text-align: center;
    line-height: 26px;
    color: var(--text-muted);
    transition: color 0.2s;
  }
  .icon.active {
    color: white;
  }
  .theme-toggle:hover .track {
    border-color: var(--border-light);
    background: var(--card-hover);
  }
</style>
