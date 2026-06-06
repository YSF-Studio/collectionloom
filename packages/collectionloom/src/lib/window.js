import { isTauri } from "./api/tauri.js";

/** @returns {Promise<import('@tauri-apps/api/window').Window | null>} */
export async function getAppWindow() {
  if (!isTauri()) return null;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

/** @returns {'macos' | 'windows' | 'linux'} */
export function guessPlatform() {
  if (typeof navigator === "undefined") return "macos";
  const ua = navigator.userAgent;
  if (/Win/i.test(ua)) return "windows";
  if (/Mac/i.test(ua)) return "macos";
  return "linux";
}
