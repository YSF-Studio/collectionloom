import { spawn } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const pkg = join(root, "packages/collectionloom");
export const PREVIEW_URL = "http://127.0.0.1:4174";

export function startFixturePreview() {
  const proc = spawn("npm", ["run", "preview", "--", "--host", "127.0.0.1", "--port", "4174", "--strictPort"], {
    cwd: pkg,
    env: { ...process.env, VITE_FIXTURE_MODE: "1" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  return proc;
}

export async function waitForPreview(proc, timeoutMs = 60000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(PREVIEW_URL);
      if (res.ok) return;
    } catch {
      /* retry */
    }
    if (proc.exitCode != null) {
      throw new Error(`Preview exited with code ${proc.exitCode}`);
    }
    await new Promise((r) => setTimeout(r, 400));
  }
  proc.kill();
  throw new Error("Preview server did not start");
}
