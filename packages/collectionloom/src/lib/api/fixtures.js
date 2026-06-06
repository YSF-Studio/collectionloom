/** Fixture backend for screenshot / preview mode — data from real processed sample files. */

let cache = null;

async function load() {
  if (cache) return cache;
  const res = await fetch("/fixtures/screenshot-data.json");
  cache = await res.json();
  return cache;
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
    return {
      hpa_dco_detection: "not_implemented",
      note: "HPA/DCO detection is not implemented. No ATA IDENTIFY DEVICE query is performed.",
    };
  }
  if (cmd === "start_disk_imaging" || cmd === "start_network_capture" || cmd === "capture_ram" || cmd === "adb_backup") {
    return "started";
  }
  if (cmd === "cancel_imaging" || cmd === "cancel_network_capture") {
    return null;
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
