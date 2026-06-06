#!/usr/bin/env node
/**
 * Assemble a portable forensic kit zip after `tauri build`.
 * Usage: node scripts/package-portable.mjs
 */
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const appDir = join(repoRoot, "packages/collectionloom");
const tauriDir = join(appDir, "src-tauri");
const releaseDir = join(tauriDir, "target/release");
const bundleDir = join(releaseDir, "bundle");
const templatesDir = join(repoRoot, "scripts/package-templates");
const toolsDir = join(repoRoot, "tools");
const distDir = join(repoRoot, "dist/portable");

const platform = process.platform;
const arch = process.arch;

function fail(msg) {
  console.error(`package-portable: ${msg}`);
  process.exit(1);
}

function readVersion() {
  const pkg = JSON.parse(readFileSync(join(appDir, "package.json"), "utf8"));
  return pkg.version || "0.0.0";
}

function osLabel() {
  if (platform === "darwin") return "macos";
  if (platform === "win32") return "windows";
  if (platform === "linux") return "linux";
  return platform;
}

function findFirstMatching(dir, predicate) {
  if (!existsSync(dir)) return null;
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (predicate(name, full)) return full;
  }
  return null;
}

function copyToolsTemplate(kitRoot) {
  const dest = join(kitRoot, "tools");
  mkdirSync(dest, { recursive: true });
  for (const file of ["README.txt", "manifest.json.example"]) {
    const src = join(toolsDir, file);
    if (!existsSync(src)) fail(`Missing repo tools/${file}`);
    copyFileSync(src, join(dest, file));
  }
}

function copyKitScaffold(kitRoot) {
  mkdirSync(join(kitRoot, "cases/acquisitions"), { recursive: true });
  writeFileSync(join(kitRoot, ".portable"), "CollectionLoom portable kit\n");
  copyFileSync(
    join(templatesDir, "README-PORTABLE.txt"),
    join(kitRoot, "README-PORTABLE.txt"),
  );
  copyToolsTemplate(kitRoot);
}

function assembleMacKit(staging) {
  const appBundle =
    findFirstMatching(join(bundleDir, "macos"), (n, p) =>
      n.endsWith(".app") && statSync(p).isDirectory(),
    ) ??
    findFirstMatching(releaseDir, (n, p) =>
      n.endsWith(".app") && statSync(p).isDirectory(),
    );

  if (!appBundle) {
    fail(
      `macOS app bundle not found under ${bundleDir}/macos — run tauri build first`,
    );
  }

  copyKitScaffold(staging);
  cpSync(appBundle, join(staging, basename(appBundle)), { recursive: true });

  const launcherSrc = join(templatesDir, "start-collectionloom.sh");
  const launcherDest = join(staging, "start-collectionloom.sh");
  copyFileSync(launcherSrc, launcherDest);
  spawnSync("chmod", ["+x", launcherDest], { stdio: "inherit" });
}

function assembleWindowsKit(staging) {
  const exe =
    findFirstMatching(releaseDir, (n) => n.toLowerCase() === "collectionloom.exe") ??
    findFirstMatching(releaseDir, (n) => n.endsWith(".exe") && n.toLowerCase().includes("collectionloom"));

  if (!exe) {
    fail(`collectionloom.exe not found in ${releaseDir} — run tauri build first`);
  }

  copyKitScaffold(staging);
  copyFileSync(exe, join(staging, "collectionloom.exe"));

  for (const name of readdirSync(releaseDir)) {
    if (!name.toLowerCase().endsWith(".dll")) continue;
    copyFileSync(join(releaseDir, name), join(staging, name));
  }

  copyFileSync(
    join(templatesDir, "Start-CollectionLoom.bat"),
    join(staging, "Start-CollectionLoom.bat"),
  );
}

function assembleLinuxKit(staging) {
  copyKitScaffold(staging);

  const appImage = findFirstMatching(join(bundleDir, "appimage"), (n, p) =>
    n.endsWith(".AppImage") && statSync(p).isFile(),
  );

  if (appImage) {
    const dest = join(staging, basename(appImage));
    copyFileSync(appImage, dest);
    spawnSync("chmod", ["+x", dest], { stdio: "inherit" });
  } else {
    const binary =
      findFirstMatching(releaseDir, (n, p) =>
        n === "collectionloom" && statSync(p).isFile(),
      ) ?? join(releaseDir, "collectionloom");
    if (!existsSync(binary)) {
      fail(
        `No AppImage in ${bundleDir}/appimage and no collectionloom binary in ${releaseDir}`,
      );
    }
    copyFileSync(binary, join(staging, "collectionloom"));
    spawnSync("chmod", ["+x", join(staging, "collectionloom")], {
      stdio: "inherit",
    });
  }

  const launcherSrc = join(templatesDir, "start-collectionloom.sh");
  const launcherDest = join(staging, "start-collectionloom.sh");
  copyFileSync(launcherSrc, launcherDest);
  spawnSync("chmod", ["+x", launcherDest], { stdio: "inherit" });
}

function createZip(stagingDir, kitFolderName, zipPath) {
  mkdirSync(dirname(zipPath), { recursive: true });
  if (existsSync(zipPath)) rmSync(zipPath);

  if (platform === "win32") {
    const ps = [
      "Compress-Archive",
      "-Path",
      join(stagingDir, kitFolderName),
      "-DestinationPath",
      zipPath,
      "-Force",
    ];
    const result = spawnSync(
      "powershell",
      ["-NoProfile", "-Command", ps.join(" ")],
      { stdio: "inherit", shell: true },
    );
    if (result.status !== 0) fail("Compress-Archive failed");
    return;
  }

  const zipCheck = spawnSync("zip", ["-v"], { encoding: "utf8" });
  if (zipCheck.status === 0) {
    const result = spawnSync(
      "zip",
      ["-r", zipPath, kitFolderName],
      { cwd: stagingDir, stdio: "inherit" },
    );
    if (result.status !== 0) fail("zip command failed");
    return;
  }

  // Fallback when zip is unavailable (some minimal Linux images)
  const result = spawnSync(
    "tar",
    ["-caf", zipPath.replace(/\.zip$/i, ".tar.gz"), "-C", stagingDir, kitFolderName],
    { stdio: "inherit" },
  );
  if (result.status !== 0) fail("tar fallback failed");
}

function main() {
  if (!existsSync(releaseDir)) {
    fail(`Release directory missing: ${releaseDir}`);
  }

  const version = readVersion();
  const zipBase = `CollectionLoom-${version}-portable-${osLabel()}-${arch}`;
  const staging = join(distDir, `.staging-${zipBase}`);
  const kitFolder = join(staging, "CollectionLoom");

  rmSync(staging, { recursive: true, force: true });
  mkdirSync(kitFolder, { recursive: true });

  if (platform === "darwin") assembleMacKit(kitFolder);
  else if (platform === "win32") assembleWindowsKit(kitFolder);
  else if (platform === "linux") assembleLinuxKit(kitFolder);
  else fail(`Unsupported platform: ${platform}`);

  let zipPath = join(distDir, `${zipBase}.zip`);
  createZip(staging, "CollectionLoom", zipPath);

  if (!existsSync(zipPath)) {
    const tarGz = zipPath.replace(/\.zip$/i, ".tar.gz");
    if (existsSync(tarGz)) zipPath = tarGz;
  }

  rmSync(staging, { recursive: true, force: true });
  console.log(`Portable kit: ${zipPath}`);
}

main();
