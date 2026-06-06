#!/usr/bin/env node
/**
 * Capture CollectionLoom UI screenshots for GitHub documentation.
 * Light mode, fixture backend, real sample file hashes (prepare-screenshot-data.mjs).
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
const ui = fixtureMeta.uiState ?? {};
const verifyHash = fixtureMeta.meta.verifyMeSha256;
const sampleDevice = ui.diskDevice ?? "/dev/disk4";

async function applyLightMode(page) {
  await page.evaluate(() => {
    localStorage.setItem("collectionloom-theme", "light");
    document.documentElement.classList.add("light-mode");
    document.body.classList.add("light-mode");
  });
}

async function selectDisk(page, device = sampleDevice) {
  const diskSelect = page.locator(".disk-tab select.full").first();
  if (await diskSelect.count()) {
    await diskSelect.selectOption(device);
    await page.waitForTimeout(600);
  }
  const wbSelect = page.locator(".wb-device-select");
  if (await wbSelect.count()) {
    await wbSelect.selectOption(device);
    await page.waitForTimeout(400);
  }
}

const sections = [
  {
    id: "disk",
    file: "collection_disk_imaging.png",
    setup: async (page) => {
      await selectDisk(page);
      await page.locator(".disk-tab button.btn-sm", { hasText: "Check HPA/DCO" }).click();
      await page.waitForTimeout(700);
      await page.locator('.disk-tab input[placeholder="/path/to/image.dd"]').fill(ui.diskDestination ?? "~/Evidence/samples/source_disk.img.dd");
      await page.locator(".disk-tab button.btn-primary", { hasText: "Start Acquisition" }).click();
      await page.waitForTimeout(1800);
      await page.locator(".summary-grid").waitFor({ timeout: 5000 }).catch(() => {});
    },
  },
  {
    id: "ram",
    file: "collection_ram_capture.png",
    setup: async (page) => {
      const tool = page.locator("select").first();
      if (await tool.count()) await tool.selectOption({ index: 1 });
      await page.locator('input[type="text"]').first().fill("~/Evidence/samples/ram_capture.lime");
      await page.locator("button", { hasText: "List Processes" }).click();
      await page.waitForTimeout(800);
    },
  },
  { id: "mobile", file: "collection_mobile_triage.png" },
  {
    id: "cloud",
    file: "collection_cloud_snapshot.png",
    setup: async (page) => {
      await page.locator("button", { hasText: "Browse" }).click();
      await page.waitForTimeout(400);
      await page.locator('input[placeholder*="vol-"]').fill(ui.cloudResourceId ?? "vol-0demo1234567890abcd");
      await page.locator("button", { hasText: "Create Snapshot" }).click();
      await page.waitForTimeout(900);
    },
  },
  {
    id: "network",
    file: "collection_network_capture.png",
    setup: async (page) => {
      await page.waitForTimeout(800);
      await page.locator(".network-tab select.full").selectOption("en0");
      await page.locator('.network-tab input[placeholder="3600"]').fill("3600");
      await page.locator(".network-tab button.btn-primary", { hasText: "Start Capture" }).click();
      await page.waitForTimeout(1500);
    },
  },
  { id: "snapshot", file: "collection_system_snapshot.png" },
  {
    id: "acquire-all",
    file: "collection_acquire_all.png",
    setup: async (page) => {
      await page.locator("button", { hasText: "Detect Devices" }).click();
      await page.waitForTimeout(900);
      const sel = page.locator("select.full").first();
      if (await sel.count()) await sel.selectOption(sampleDevice).catch(() => {});
    },
  },
  { id: "encryption", file: "collection_encryption.png" },
  {
    id: "verify",
    file: "collection_hash_verify.png",
    setup: async (page) => {
      await page.fill('input[placeholder="/path/to/evidence.dd"]', ui.verifyFilePath ?? "/fixtures/samples/verify_me.txt");
      await page.fill(".hash-input", verifyHash);
      await page.locator("button.btn-primary", { hasText: "Verify" }).click();
      await page.waitForTimeout(800);
    },
  },
  { id: "dashboard", file: "collection_case_dashboard.png" },
  {
    id: "coc",
    file: "collection_chain_of_custody.png",
    setup: async (page) => {
      await page.locator("#coc-title").fill(ui.cocCaseName ?? "Incident Response — Workstation Triage");
      await page.locator("#coc-operator").fill(ui.cocOperator ?? "J. Analyst");
      await page.locator("#coc-device").fill(ui.cocDevice ?? sampleDevice);
      await page.locator("button.btn-primary", { hasText: "Create Chain of Custody" }).click();
      await page.waitForTimeout(1200);
      await page.locator(".qr-img, .evidence-id").first().waitFor({ timeout: 5000 }).catch(() => {});
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
  console.log("Starting preview server with fixture mode (light theme)…");
  const proc = startPreview();
  proc.stderr?.on("data", (d) => process.stderr.write(d));
  await waitForServer(proc);

  const browser = await chromium.launch();
  const context = await browser.newContext({
    viewport: { width: 1280, height: 860 },
    colorScheme: "light",
  });
  await context.addInitScript(() => {
    localStorage.setItem("collectionloom-theme", "light");
  });
  const page = await context.newPage();

  for (const sec of sections) {
    console.log(`Capturing ${sec.id}…`);
    await page.goto(`http://127.0.0.1:4174/#${sec.id}`, { waitUntil: "networkidle" });
    await applyLightMode(page);
    await page.waitForTimeout(700);
    if (sec.setup) await sec.setup(page);
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.waitForTimeout(300);
    await page.screenshot({
      path: join(outDir, sec.file),
      fullPage: false,
    });
  }

  await browser.close();
  proc.kill();
  console.log(`Done — ${sections.length} light-mode screenshots in ${outDir}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
