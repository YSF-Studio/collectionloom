/** Portable forensic kit paths — cross-platform (Windows / Linux / macOS). */

import { invoke, isTauri } from "./tauri.js";

/** @typedef {{ platform: string, kitRoot?: string, toolsDir?: string, casesDir: string, defaultAcquisitionDir: string, portableMode: boolean, pathSeparator: string }} PortableLayout */

/** @returns {Promise<PortableLayout>} */
export async function getPortableLayout() {
  if (!isTauri()) {
    return {
      platform: "preview",
      casesDir: "~/CollectionLoom/cases",
      defaultAcquisitionDir: "/tmp/collectionloom_acquisition",
      portableMode: false,
      pathSeparator: "/",
    };
  }
  return invoke("get_portable_layout");
}

/** @param {string} base @param {...string} parts */
export function joinPortablePath(base, ...parts) {
  let p = base.replace(/[\\/]+$/, "");
  const sep = p.includes("\\") ? "\\" : "/";
  for (const part of parts) {
    p = `${p}${sep}${part.replace(/^[/\\]+/, "")}`;
  }
  return p;
}

/** @param {string} filename @returns {Promise<string>} */
export async function defaultOutputPath(filename) {
  const layout = await getPortableLayout();
  return joinPortablePath(layout.defaultAcquisitionDir, filename);
}
