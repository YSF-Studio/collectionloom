const STORAGE_KEY = "collectionloom-theme";

/** @type {"dark" | "light"} */
function readStored() {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark") return v;
  } catch {
    /* ignore */
  }
  return "dark";
}

/** @param {"dark" | "light"} mode */
export function applyTheme(mode) {
  const root = document.documentElement;
  const body = document.body;
  if (mode === "light") {
    root.classList.add("light-mode");
    body.classList.add("light-mode");
  } else {
    root.classList.remove("light-mode");
    body.classList.remove("light-mode");
  }
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
}

export function initTheme() {
  applyTheme(readStored());
}

/** @returns {"dark" | "light"} */
export function getTheme() {
  return document.documentElement.classList.contains("light-mode") ? "light" : "dark";
}

/** @param {"dark" | "light"} mode */
export function setTheme(mode) {
  applyTheme(mode);
}

/** Toggle and return new mode */
export function toggleTheme() {
  const next = getTheme() === "dark" ? "light" : "dark";
  applyTheme(next);
  return next;
}
