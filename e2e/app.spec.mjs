#!/usr/bin/env node
/**
 * GUI E2E (fixture mode): opens real browser, navigates tabs, fails on console errors.
 * Full Tauri IPC is tested when running `tauri dev` + DevTools; this catches UI regressions in CI.
 */
import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { PREVIEW_URL, startFixturePreview, waitForPreview } from "./helpers/preview-server.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = join(root, "packages/collectionloom");

async function buildFixture() {
  await new Promise((resolve, reject) => {
    const proc = spawn("npm", ["run", "build"], {
      cwd: pkg,
      env: { ...process.env, VITE_FIXTURE_MODE: "1" },
      stdio: "inherit",
    });
    proc.on("exit", (code) => (code === 0 ? resolve() : reject(new Error(`build exit ${code}`))));
  });
}

const SIDEBAR_TABS = [
  "Disk Imaging",
  "RAM Capture",
  "Mobile Triage",
  "Prerequisites",
  "About",
];

async function main() {
  console.log("[e2e] Building fixture frontend…");
  await buildFixture();

  console.log("[e2e] Starting preview server…");
  const proc = startFixturePreview();
  proc.stderr?.on("data", (d) => process.stderr.write(d));
  await waitForPreview(proc);

  const consoleErrors = [];
  const pageErrors = [];

  const browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1280, height: 860 } });
  const page = await context.newPage();

  page.on("console", (msg) => {
    if (msg.type() === "error") {
      const text = msg.text();
      if (!text.includes("favicon")) consoleErrors.push(text);
    }
  });
  page.on("pageerror", (err) => pageErrors.push(String(err)));

  await page.goto(PREVIEW_URL, { waitUntil: "networkidle" });
  await page.waitForSelector(".sidebar-item", { timeout: 15000 });

  for (const label of SIDEBAR_TABS) {
    console.log(`[e2e] Tab: ${label}`);
    await page.locator(".sidebar-item", { hasText: label }).click();
    await page.waitForTimeout(600);
    await page.waitForSelector(".tab-content, .preflight-tab, .app-shell", { timeout: 8000 });
  }

  // Disk tab: sidebar disk list + refresh
  await page.locator(".sidebar-item", { hasText: "Disk Imaging" }).click();
  await page.waitForTimeout(400);
  const refresh = page.locator(".disk-sidebar button", { hasText: "Refresh" });
  if (await refresh.count()) {
    await refresh.click();
    await page.waitForTimeout(800);
  }
  const diskItem = page.locator(".disk-item").first();
  if (await diskItem.count()) {
    await diskItem.click();
    await page.waitForTimeout(500);
  }

  // RAM tab
  await page.locator(".sidebar-item", { hasText: "RAM Capture" }).click();
  await page.waitForTimeout(500);
  const ramRefresh = page.locator(".tab-content button", { hasText: "Refresh" }).first();
  if (await ramRefresh.count()) await ramRefresh.click();

  await browser.close();
  proc.kill();

  const failures = [...pageErrors, ...consoleErrors].filter(Boolean);
  if (failures.length) {
    console.error("[e2e] FAILED — console/page errors:");
    failures.forEach((e) => console.error(" ", e));
    process.exit(1);
  }

  console.log(`[e2e] PASSED — ${SIDEBAR_TABS.length} tabs, no console errors.`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
