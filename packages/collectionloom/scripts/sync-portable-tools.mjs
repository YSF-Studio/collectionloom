#!/usr/bin/env node
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const sourceTools = join(repoRoot, "tools");
const targetTools = join(repoRoot, "packages/collectionloom/src-tauri/resources/tools");

function main() {
  mkdirSync(targetTools, { recursive: true });

  for (const entry of readdirSync(targetTools)) {
    if (entry === ".gitkeep" || entry === "README.txt" || entry === "manifest.template.json") continue;
    rmSync(join(targetTools, entry), { recursive: true, force: true });
  }

  if (!existsSync(sourceTools)) {
    console.log("No repo tools/ directory found; portable bundle will use bundled resources only.");
    return;
  }

  for (const entry of readdirSync(sourceTools)) {
    const src = join(sourceTools, entry);
    const dest = join(targetTools, entry);
    cpSync(src, dest, { recursive: true });
  }

  console.log(`Synced ${sourceTools} -> ${targetTools}`);
}

main();
