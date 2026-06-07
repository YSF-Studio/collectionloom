<script>
  import { getTheme, setTheme } from "../../stores/theme.js";

  let mode = $state(getTheme());

  function choose(next) {
    setTheme(next);
    mode = next;
  }
</script>

<span class="theme-toggle" role="group" aria-label={`Theme: ${mode}`}>
  <button class="seg" type="button" class:active={mode === "light"} onclick={() => choose("light")}>L</button>
  <button class="seg" type="button" class:active={mode === "dark"} onclick={() => choose("dark")}>D</button>
  <button class="seg sys" type="button" class:active={mode === "system"} onclick={() => choose("system")}>S</button>
</span>

<style>
  .theme-toggle {
    -webkit-app-region: no-drag;
    display: inline-flex;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: 13px;
    overflow: hidden;
    background: var(--card);
  }
  .seg {
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    padding: 0;
    transition: background 0.2s, color 0.2s;
  }
  .seg:hover { background: var(--card-hover); }
  .seg.active {
    background: var(--primary);
    color: white;
  }
  .seg.sys {
    width: 26px;
  }
</style>
