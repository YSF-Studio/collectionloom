<script>
  import { getLocale, setLocale } from "../../stores/locale.js";

  let mode = $state(getLocale());

  function choose(next) {
    setLocale(next);
    mode = next;
  }
</script>

<span class="locale-toggle" role="group" aria-label={`Language: ${mode}`}>
  <button class="seg" type="button" class:active={mode === "en"} onclick={() => choose("en")}>EN</button>
  <button class="seg" type="button" class:active={mode === "id"} onclick={() => choose("id")}>ID</button>
  <button class="seg sys" type="button" class:active={mode === "system"} onclick={() => choose("system")}>S</button>
</span>

<style>
  .locale-toggle {
    -webkit-app-region: no-drag;
    display: inline-flex;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: 13px;
    overflow: hidden;
    background: var(--card);
  }
  .seg {
    width: 28px;
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
