#!/usr/bin/env node
/**
 * Fail if frontend invoke('cmd') calls are not registered in lib.rs generate_handler!.
 */
import { readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const libRs = readFileSync(
  join(root, "packages/collectionloom/src-tauri/src/lib.rs"),
  "utf8"
);
const srcDir = join(root, "packages/collectionloom/src");

function registeredCommands() {
  const m = libRs.match(/generate_handler!\[\s*([\s\S]*?)\s*\]/);
  if (!m) throw new Error("generate_handler! not found in lib.rs");
  const names = new Set();
  for (const line of m[1].split(",")) {
    const part = line.trim().split("::").pop();
    if (part) names.add(part);
  }
  return names;
}

function walk(dir, out = []) {
  for (const ent of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else if (/\.(js|svelte|ts)$/.test(ent.name)) out.push(p);
  }
  return out;
}

function frontendInvokes() {
  const re = /invoke\s*\(\s*["'`]([a-z0-9_]+)["'`]/g;
  const names = new Set();
  for (const file of walk(srcDir)) {
    const text = readFileSync(file, "utf8");
    let match;
    while ((match = re.exec(text)) !== null) {
      names.add(match[1]);
    }
  }
  return names;
}

const registered = registeredCommands();
const invoked = frontendInvokes();
const missing = [...invoked].filter((c) => !registered.has(c)).sort();

if (missing.length) {
  console.error("IPC audit FAILED — frontend invoke() not in generate_handler!:");
  for (const c of missing) console.error(`  - ${c}`);
  console.error("\nAdd to packages/collectionloom/src-tauri/src/lib.rs generate_handler![...]");
  process.exit(1);
}

console.log(`IPC audit OK — ${invoked.size} invoke(s), ${registered.size} handler(s) registered.`);
