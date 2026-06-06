/** Fixture backend for screenshot / preview mode — data from real processed sample files. */

let cache = null;
let imagingStarted = false;
let imagingPolls = 0;
let networkCapturing = false;

async function load() {
  if (cache) return cache;
  const res = await fetch("/fixtures/screenshot-data.json");
  cache = await res.json();
  return cache;
}

function uiState(data) {
  return data.uiState ?? {};
}

/** @param {string} cmd @param {Record<string, unknown>} args */
export async function fixtureInvoke(cmd, args = {}) {
  const data = await load();
  const table = data.commands ?? {};

  if (cmd === "verify_hash") {
    return table.verify_hash;
  }
  if (cmd === "check_write_blocker") {
    return table.check_write_blocker;
  }
  if (cmd === "enable_write_blocker" || cmd === "disable_write_blocker") {
    const base = table.check_write_blocker ?? {};
    return cmd === "enable_write_blocker"
      ? { ...base, active: true, enabled: true, software: true, method: "software", notes: "Software write-blocker enabled (preview)." }
      : { ...base, active: false, enabled: false, software: false, method: "none", notes: "Write-blocker disabled (preview)." };
  }
  if (cmd === "create_case") {
    return {
      case_id: "CL-2026-NEW",
      title: args.title ?? "New Case",
      operator: args.operator ?? "Investigator",
      timezone: args.timezone ?? "UTC",
      status: "open",
      created_at: new Date().toISOString(),
    };
  }
  if (cmd === "compare_snapshots") {
    return {
      added: [{ key: "process:9012", domain: "process", summary: "Google Chrome (new)" }],
      removed: [],
      changed: [{ key: "network:443", domain: "network", summary: "New TLS connection" }],
    };
  }
  if (cmd === "export_json" || cmd === "export_markdown" || cmd === "export_zip") {
    return {
      export_id: "exp-demo",
      case_id: args.caseId ?? "CL-2026-DEMO",
      format: cmd.replace("export_", ""),
      path: "~/CollectionLoom/cases/CL-2026-DEMO/exports/demo." + (cmd === "export_zip" ? "zip" : cmd === "export_markdown" ? "md" : "json"),
      created_at: new Date().toISOString(),
    };
  }
  if (cmd === "generate_qr_label") {
    const png = await fetch("/icon.png").then((r) => r.arrayBuffer());
    return Array.from(new Uint8Array(png));
  }
  if (cmd === "sign_coc") {
    return { signature: "demo-ed25519-signature", publicKey: "demo-public-key-base64" };
  }
  if (cmd === "create_chain_of_custody") {
    return { evidenceId: table.generate_evidence_id, status: "created" };
  }
  if (cmd === "generate_coc_report") {
    return "~/CollectionLoom/cases/CL-2026-DEMO/coc_report.pdf";
  }
  if (cmd === "compute_file_hash") {
    return table.verify_hash?.actual ?? "";
  }
  if (cmd === "hpa_dco_detect") {
    return table.hpa_dco_detect ?? {
      device: args.device ?? "/dev/disk4",
      supported: true,
      hpaDetected: false,
      dcoDetected: false,
      identifyMaxLba: null,
      nativeMaxLba: null,
      dcoMaxLba: null,
      hiddenSectors: null,
      model: "Sample USB Evidence Drive",
      notes: "Fixture mode — no ATA pass-through",
    };
  }
  if (cmd === "pick_cloud_credentials") {
    return uiState(data).cloudCredentialPath ?? "/fixtures/samples/demo-aws-credentials.json";
  }
  if (cmd === "create_cloud_snapshot") {
    return table.create_cloud_snapshot ?? { snapshot_id: "snap-0demo1234567890ab", status: "completed" };
  }
  if (cmd === "get_imaging_progress") {
    if (imagingStarted) {
      imagingPolls += 1;
      if (imagingPolls >= 2) {
        imagingStarted = false;
        return table.get_imaging_progress_done ?? table.get_imaging_progress;
      }
      return table.get_imaging_progress;
    }
    return table.get_imaging_progress_done ?? table.get_imaging_progress;
  }
  if (cmd === "start_disk_imaging") {
    imagingStarted = true;
    imagingPolls = 0;
    return "started";
  }
  if (cmd === "start_network_capture") {
    networkCapturing = true;
    return "started";
  }
  if (cmd === "get_capture_stats") {
    if (networkCapturing) return table.get_capture_stats;
    return { packets: 0, bytes: 0, bytes_captured: 0 };
  }
  if (cmd === "get_capture_packets") {
    if (networkCapturing) return table.get_capture_packets;
    return [];
  }
  if (cmd === "cancel_network_capture") {
    networkCapturing = false;
    return null;
  }
  if (cmd === "cancel_imaging") {
    imagingStarted = false;
    imagingPolls = 0;
    return null;
  }
  if (cmd === "capture_ram" || cmd === "adb_backup") {
    return "started";
  }
  if (cmd === "start_snapshot") {
    return table.list_snapshots_cmd?.[0] ?? {};
  }
  if (cmd === "get_case") {
    return table.list_cases_cmd?.[0] ?? {};
  }
  if (Object.prototype.hasOwnProperty.call(table, cmd)) {
    return table[cmd];
  }
  return null;
}

export function isFixtureMode() {
  return import.meta.env.VITE_FIXTURE_MODE === "1";
}
