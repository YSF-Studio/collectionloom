#!/usr/bin/env node
/**
 * Download third-party forensic tools into the repo-level `tools/` folder,
 * then mirror them into Tauri bundle resources.
 *
 * Run before `tauri build`. The repo `tools/` directory is the source of truth
 * for portable/manual staging; `packages/collectionloom/src-tauri/resources/tools/`
 * is what the app bundle consumes at build time.
 *
 * SKIP_TOOL_DOWNLOAD=1     — skip network fetch (dev/CI without downloads)
 * STRICT_TOOL_DOWNLOAD=1   — fail build if any tool download fails
 * COLLECTIONLOOM_BUILD_FLAVOR=source|portable|commercial — labels manifest metadata
 */

import { createHash } from "node:crypto";
import { chmod, mkdirSync, readFileSync, readdirSync, writeFileSync, cpSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const SOURCE_TOOLS = join(ROOT, "tools");
const RES_TOOLS = join(ROOT, "packages/collectionloom/src-tauri/resources/tools");
const CONFIG_PATH = join(__dirname, "tools.config.json");

const SKIP = process.env.SKIP_TOOL_DOWNLOAD === "1";
const STRICT = process.env.STRICT_TOOL_DOWNLOAD === "1";
const BUILD_FLAVOR = process.env.COLLECTIONLOOM_BUILD_FLAVOR || "source";

function platformKey() {
  const p = process.platform;
  const a = process.arch === "arm64" ? "arm64" : "x64";
  if (p === "darwin") return `darwin-${a}`;
  if (p === "linux") return `linux-${a}`;
  if (p === "win32") return `win32-${a}`;
  return `${p}-${a}`;
}

async function sha256File(path) {
  const data = await readFileSync(path);
  return createHash("sha256").update(data).digest("hex");
}

function syncContents(srcDir, destDir) {
  mkdirSync(destDir, { recursive: true });
  for (const entry of readdirSync(srcDir)) {
    cpSync(join(srcDir, entry), join(destDir, entry), { recursive: true });
  }
}

async function download(url, dest) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) {
    throw new Error(`HTTP ${res.status} for ${url}`);
  }
  await mkdirSync(dirname(dest), { recursive: true });
  const body = Buffer.from(await res.arrayBuffer());
  writeFileSync(dest, body);
  if (process.platform !== "win32") {
    await chmod(dest, 0o755);
  }
}

async function main() {
  await mkdirSync(SOURCE_TOOLS, { recursive: true });
  await mkdirSync(RES_TOOLS, { recursive: true });

  if (SKIP) {
    console.log("[download-tools] SKIP_TOOL_DOWNLOAD=1 — keeping existing resources/tools/");
    const template = join(SOURCE_TOOLS, "manifest.json");
    const manifest = join(RES_TOOLS, "manifest.json");
    try {
      await writeFileSync(manifest, readFileSync(template));
    } catch {
      await writeFileSync(manifest, '{"tools":{}}\n');
    }
    syncContents(SOURCE_TOOLS, RES_TOOLS);
    return;
  }

  const config = JSON.parse(readFileSync(CONFIG_PATH, "utf8"));
  const pk = platformKey();
  console.log(`[download-tools] Platform: ${pk}`);

  /** @type {Record<string, { file: string, sha256: string, release?: string }>} */
  const manifest = { buildFlavor: BUILD_FLAVOR, tools: {} };

  for (const tool of config.tools) {
    const assetKeys = Object.keys(tool.assets);
    const asset = tool.assets[pk];
    if (!asset) {
      const supported = assetKeys.length ? assetKeys.join(", ") : "manual/source-specific";
      console.log(`[download-tools] Note ${tool.id} is not downloadable on ${pk} (${supported})`);
      if (tool.id === "lime" || tool.id === "mrs") {
        const noteDir = join(SOURCE_TOOLS, tool.id);
        mkdirSync(noteDir, { recursive: true });
        writeFileSync(
          join(noteDir, "README.txt"),
          `${tool.id.toUpperCase()} is source-specific and has no official download artifact for ${pk}.\n` +
            `Stage a compatible build manually in this folder when needed.\n`,
        );
      }
      continue;
    }

    const dest = join(SOURCE_TOOLS, asset.file);
    try {
      console.log(`[download-tools] Fetch ${tool.id} → ${asset.file}`);
      await download(asset.url, dest);
      const hash = await sha256File(dest);
      const keys = [tool.id, ...(asset.aliases ?? [])];
      for (const key of keys) {
        manifest.tools[key] = { file: asset.file, sha256: hash, release: tool.release };
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

  writeFileSync(join(SOURCE_TOOLS, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  syncContents(SOURCE_TOOLS, RES_TOOLS);
  console.log("[download-tools] Done → tools/ and packages/collectionloom/src-tauri/resources/tools/");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
