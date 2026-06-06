import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const root = path.resolve(__dirname, "..");
const appDir = path.join(root, "packages/collectionloom");

const tauriDriverPath =
  process.env.TAURI_DRIVER_PATH ??
  path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver");

function appBinary() {
  const base = path.join(appDir, "src-tauri", "target", "debug", "collectionloom");
  if (process.platform === "win32") return `${base}.exe`;
  return base;
}

let tauriDriver;
let exit = false;

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };
  for (const sig of ["exit", "SIGINT", "SIGTERM", "SIGHUP", "SIGBREAK"]) {
    process.on(sig, cleanup);
  }
}

onShutdown(() => closeTauriDriver());

export const config = {
  hostname: "127.0.0.1",
  port: 4444,
  specs: ["./specs/**/*.spec.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: appBinary(),
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 120000,
  },

  onPrepare: () => {
    if (process.env.SKIP_TAURI_BUILD === "1") {
      console.log("[e2e-webdriver] SKIP_TAURI_BUILD=1 — using existing debug binary");
      return;
    }
    console.log("[e2e-webdriver] Building frontend + Tauri debug binary…");
    spawnSync("npm", ["run", "build"], {
      cwd: appDir,
      stdio: "inherit",
      shell: true,
      env: { ...process.env, SKIP_TOOL_DOWNLOAD: process.env.SKIP_TOOL_DOWNLOAD ?? "1" },
    });
    const build = spawnSync(
      "npm",
      ["run", "tauri", "--", "build", "--", "--debug", "--no-bundle"],
      { cwd: appDir, stdio: "inherit", shell: true, env: process.env },
    );
    if (build.status !== 0) {
      throw new Error(`tauri build --debug failed with code ${build.status}`);
    }
  },

  beforeSession: () => {
    console.log(`[e2e-webdriver] Starting tauri-driver (${tauriDriverPath})…`);
    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });
    tauriDriver.on("error", (error) => {
      console.error("tauri-driver error:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code) => {
      if (!exit) {
        console.error("tauri-driver exited with code:", code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    closeTauriDriver();
  },
};
