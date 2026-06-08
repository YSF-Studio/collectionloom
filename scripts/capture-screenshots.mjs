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

async function selectOptionSafely(locator, preferredValue) {
  const optionValues = await locator.locator("option").evaluateAll((options) =>
    options
      .map((option) => option.value)
      .filter((value) => value !== "")
  ).catch(() => []);

  if (!optionValues.length) return false;

  const valueToUse = optionValues.includes(preferredValue)
    ? preferredValue
    : optionValues[0];

  await locator.selectOption(valueToUse);
  return true;
}

async function selectDisk(page, device = sampleDevice) {
  const refresh = page.locator(".disk-tab button.btn-sm", {
    hasText: /Refresh|Segarkan/i,
  }).first();
  if (await refresh.count()) {
    await refresh.click();
    await page.locator(".disk-tab .disk-item").first().waitFor({ state: "visible", timeout: 10000 }).catch(() => {});
  }

  const diskButtons = page.locator(".disk-tab .disk-item");
  const count = await diskButtons.count();
  if (!count) return;

  const targetIndex = await diskButtons.evaluateAll((buttons, preferred) => {
    const preferredIndex = buttons.findIndex((button) => button.textContent?.includes(preferred));
    return preferredIndex >= 0 ? preferredIndex : 0;
  }, device);

  await diskButtons.nth(targetIndex).click();
  await page.waitForFunction(() => {
    const btn = document.querySelector(".disk-tab button.btn-primary");
    return Boolean(btn && !btn.disabled);
  }, { timeout: 10000 }).catch(() => {});
}

const sections = [
  {
    id: "disk",
    file: "collection_disk_imaging.png",
    setup: async (page) => {
      await selectDisk(page);
      const hpaButton = page.locator(".disk-tab button.btn-sm", {
        hasText: /Check HPA\/DCO|Periksa HPA\/DCO|HPA\/DCO/i,
      });
      if (await hpaButton.count()) {
        await hpaButton.click();
        await page.waitForTimeout(700);
      }
      await page.locator('.disk-tab input[placeholder="/path/to/image.dd"]').fill(ui.diskDestination ?? "~/Evidence/samples/source_disk.img.dd");
      await page.waitForTimeout(700);
    },
  },
  {
    id: "ram",
    file: "collection_ram_capture.png",
    setup: async (page) => {
      const ramTab = page.locator(".tab-content").filter({
        hasText: /RAM Capture|Tangkap RAM/i,
      }).first();
      if (await ramTab.locator(".apple-note").count()) {
        await page.waitForTimeout(800);
        return;
      }
      const tool = ramTab.locator("select").first();
      if (await tool.count()) await tool.selectOption({ index: 1 }).catch(() => {});
      const outputInput = ramTab.locator('input[type="text"]').first();
      if (await outputInput.count()) {
        await outputInput.fill("~/Evidence/samples/ram_capture.lime").catch(() => {});
      }
      const listBtn = ramTab.locator("button", { hasText: /List Processes|Daftar Proses/i });
      if (await listBtn.count()) {
        await listBtn.click().catch(() => {});
      }
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
      await selectOptionSafely(page.locator(".network-tab select.full"), "en0");
      await page.locator('.network-tab input[placeholder="3600"]').fill("3600");
      await page.waitForTimeout(900);
    },
  },
  { id: "snapshot", file: "collection_system_snapshot.png" },
  {
    id: "acquire-all",
    file: "collection_acquire_all.png",
    setup: async (page) => {
      await page.locator("button", { hasText: "Detect Sources" }).click();
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
