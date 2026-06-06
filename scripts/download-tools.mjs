#!/usr/bin/env node
/**
 * Download third-party forensic tools into Tauri bundle resources.
 * Run before `tauri build` — tools ship inside the app (no separate tools/ folder).
 *
 * SKIP_TOOL_DOWNLOAD=1     — skip network fetch (dev/CI without downloads)
 * STRICT_TOOL_DOWNLOAD=1   — fail build if any tool download fails
 */

import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const RES_TOOLS = join(ROOT, "packages/collectionloom/src-tauri/resources/tools");
const CONFIG_PATH = join(__dirname, "tools.config.json");

const SKIP = process.env.SKIP_TOOL_DOWNLOAD === "1";
const STRICT = process.env.STRICT_TOOL_DOWNLOAD === "1";

function platformKey() {
  const p = process.platform;
  const a = process.arch === "arm64" ? "arm64" : "x64";
  if (p === "darwin") return `darwin-${a}`;
  if (p === "linux") return `linux-${a}`;
  if (p === "win32") return `win32-${a}`;
  return `${p}-${a}`;
}

async function sha256File(path) {
  const data = await readFile(path);
  return createHash("sha256").update(data).digest("hex");
}

async function download(url, dest) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status} for ${url}`);
  }
  await mkdir(dirname(dest), { recursive: true });
  const body = Readable.fromWeb(res.body);
  await pipeline(body, createWriteStream(dest));
  if (process.platform !== "win32") {
    await chmod(dest, 0o755);
  }
}

async function main() {
  await mkdir(RES_TOOLS, { recursive: true });

  if (SKIP) {
    console.log("[download-tools] SKIP_TOOL_DOWNLOAD=1 — keeping existing resources/tools/");
    const template = join(RES_TOOLS, "manifest.template.json");
    const manifest = join(RES_TOOLS, "manifest.json");
    try {
      await writeFile(manifest, await readFile(template));
    } catch {
      await writeFile(manifest, '{"tools":{}}\n');
    }
    return;
  }

  const config = JSON.parse(await readFile(CONFIG_PATH, "utf8"));
  const pk = platformKey();
  console.log(`[download-tools] Platform: ${pk}`);

  /** @type {Record<string, { file: string, sha256: string }>} */
  const manifest = { tools: {} };

  for (const tool of config.tools) {
    const asset = tool.assets[pk];
    if (!asset) {
      console.log(`[download-tools] Skip ${tool.id} (no asset for ${pk})`);
      continue;
    }

    const dest = join(RES_TOOLS, asset.file);
    try {
      console.log(`[download-tools] Fetch ${tool.id} → ${asset.file}`);
      await download(asset.url, dest);
      const hash = await sha256File(dest);
      const keys = [tool.id, ...(asset.aliases ?? [])];
      for (const key of keys) {
        manifest.tools[key] = { file: asset.file, sha256: hash };
      }
      console.log(`[download-tools] OK ${tool.id} (${hash.slice(0, 12)}…)`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.warn(`[download-tools] WARN ${tool.id}: ${msg}`);
      if (STRICT) {
        process.exitCode = 1;
        throw err;
      }
    }
  }

  await writeFile(join(RES_TOOLS, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log("[download-tools] Done → packages/collectionloom/src-tauri/resources/tools/");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
