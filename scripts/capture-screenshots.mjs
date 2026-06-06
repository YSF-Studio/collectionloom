#!/usr/bin/env node
/**
 * Capture CollectionLoom UI screenshots for GitHub documentation.
 * Uses fixture mode backed by real sample file hashes (see prepare-screenshot-data.mjs).
 */
import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = join(root, "packages/collectionloom");
const outDir = join(root, "screenshots");

mkdirSync(outDir, { recursive: true });

const fixtureMeta = JSON.parse(
  readFileSync(join(pkg, "public/fixtures/screenshot-data.json"), "utf8")
);
const verifyHash = fixtureMeta.meta.verifyMeSha256;

const sections = [
  { id: "disk", file: "collection_disk_imaging.png", setup: async (page) => {} },
  { id: "ram", file: "collection_ram_capture.png" },
  { id: "mobile", file: "collection_mobile_triage.png" },
  { id: "cloud", file: "collection_cloud_snapshot.png" },
  { id: "network", file: "collection_network_capture.png" },
  { id: "snapshot", file: "collection_system_snapshot.png" },
  { id: "acquire-all", file: "collection_acquire_all.png" },
  { id: "encryption", file: "collection_encryption.png" },
  {
    id: "verify",
    file: "collection_hash_verify.png",
    setup: async (page) => {
      await page.fill('input[placeholder="/path/to/evidence.dd"]', "/samples/verify_me.txt");
      await page.fill(".hash-input", verifyHash);
      await page.click("button.btn-primary");
      await page.waitForTimeout(600);
    },
  },
  { id: "dashboard", file: "collection_case_dashboard.png" },
  {
    id: "coc",
    file: "collection_chain_of_custody.png",
    setup: async (page) => {
      const inputs = page.locator("input");
      if (await inputs.count() >= 3) {
        await inputs.nth(0).fill("EV-2026-0606-A1B2C3");
        await inputs.nth(1).fill("CL-2026-DEMO");
        await inputs.nth(2).fill("J. Analyst");
      }
    },
  },
  { id: "export", file: "collection_export_bundle.png" },
  { id: "about", file: "collection_about.png" },
];

function startPreview() {
  return spawn("npm", ["run", "preview", "--", "--port", "4174", "--host", "127.0.0.1"], {
    cwd: pkg,
    env: { ...process.env, VITE_FIXTURE_MODE: "1" },
    stdio: ["ignore", "pipe", "pipe"],
  });
}

async function waitForServer(proc, timeoutMs = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch("http://127.0.0.1:4174/");
      if (res.ok) return;
    } catch {
      /* retry */
    }
    await new Promise((r) => setTimeout(r, 400));
  }
  proc.kill();
  throw new Error("Preview server did not start");
}

async function main() {
  console.log("Starting preview server with fixture mode…");
  const proc = startPreview();
  proc.stderr?.on("data", (d) => process.stderr.write(d));
  await waitForServer(proc);

  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1100, height: 720 } });

  for (const sec of sections) {
    console.log(`Capturing ${sec.id}…`);
    await page.goto(`http://127.0.0.1:4174/#${sec.id}`, { waitUntil: "networkidle" });
    await page.waitForTimeout(800);
    if (sec.setup) await sec.setup(page);
    await page.waitForTimeout(400);
    await page.screenshot({
      path: join(outDir, sec.file),
      fullPage: false,
    });
  }

  await browser.close();
  proc.kill();
  console.log(`Done — ${sections.length} screenshots in ${outDir}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
