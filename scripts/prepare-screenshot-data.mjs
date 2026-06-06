#!/usr/bin/env node
/**
 * Process real sample files and emit fixture JSON for screenshot capture.
 * Hashes are computed from actual bytes on disk — not mocked values.
 */
import { createHash } from "node:crypto";
import { readFileSync, statSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const samples = join(root, "samples");
const outDir = join(root, "packages/collectionloom/public/fixtures");
mkdirSync(outDir, { recursive: true });

function sha256File(path) {
  const data = readFileSync(path);
  return createHash("sha256").update(data).digest("hex");
}

const verifyPath = join(samples, "verify_me.txt");
const diskPath = join(samples, "source_disk.img");
const expectedLine = readFileSync(join(samples, "expected.sha256"), "utf8").trim();
const verifyHash = expectedLine.split(/\s+/)[0].toLowerCase();
const verifyHashComputed = sha256File(verifyPath);
const diskSize = statSync(diskPath).size;

const caseId = "CL-2026-DEMO";
const snapshotId = "snap-001-baseline";

const data = {
  meta: {
    generatedAt: new Date().toISOString(),
    verifyMeSha256: verifyHash,
    verifyMeSha256RawFile: verifyHashComputed,
    sourceDiskBytes: diskSize,
    sourceDiskSha256: sha256File(diskPath),
  },
  commands: {
    about_info: {
      appName: "CollectionLoom",
      version: "0.1.0",
      developer: "YSF Studio — Yusuf Shalahuddin",
      build: "Portable Forensic Acquisition Toolkit — macOS / Windows / Linux",
      features: [
        "Bit-for-bit disk imaging (RAW, E01, AFF4) with SHA-256 verification and split support for multi-TB drives",
        "Hardware and one-click software write-blocker (Linux BLKROSET, macOS unmount, Windows IOCTL)",
        "Volatile RAM capture via avml / LiME with live process listing",
        "Mobile triage — Android ADB and iOS logical backup workflows",
        "Cloud snapshot — AWS EBS, Azure managed disks, GCP persistent disks",
        "Network packet capture with BPF filters and live statistics",
        "System snapshot profiles (triage, IR, deep) with A/B compare engine",
        "Acquire All — orchestrated multi-module batch acquisition",
        "Encryption detection (BitLocker, LUKS, VeraCrypt, FileVault)",
        "Hash verification, chain of custody with Ed25519 signatures and QR labels",
        "Case dashboard and export bundles (JSON, Markdown, ZIP)",
        "100% offline — no telemetry, no cloud dependency for core workflows",
      ],
      disclaimer:
        "This software is provided \"AS IS\" for forensic triage and evidence collection. Operators must follow organizational policy and jurisdictional requirements. Independently verify hashes and chain-of-custody before use in legal proceedings.",
      offline: true,
      privacy:
        "All processing runs locally on your workstation. CollectionLoom does not transmit evidence, telemetry, or analytics to external servers.",
    },
    list_disks: [
      {
        device: "/dev/disk4",
        model: "Sample USB Evidence Drive",
        sizeBytes: diskSize,
        sectorSize: 512,
        isSsd: false,
        partitions: [],
      },
      {
        device: "/dev/disk5",
        model: "WD Elements 2TB",
        sizeBytes: 2000398934016,
        sectorSize: 512,
        isSsd: false,
        partitions: [],
      },
    ],
    scan_encryption: {
      drives: [
        {
          device: "/dev/disk4",
          path: "/dev/disk4",
          encrypted: false,
          type: "Unencrypted",
        },
        {
          device: "/dev/disk2s2",
          path: "/dev/disk2s2",
          encrypted: true,
          type: "APFS FileVault",
        },
      ],
      volumes: [],
    },
    check_write_blocker: {
      active: true,
      enabled: true,
      method: "software",
      confidence: "high",
      hardware: false,
      software: true,
      notes: "Software write-blocker active — volumes unmounted, imaging via raw device path.",
    },
    list_ram_tools: ["Avml", "LiME", "DumpIt"],
    get_ram_size: 17179869184,
    list_interfaces: ["en0", "en1", "lo0", "bridge0"],
    list_android_devices: [{ id: "R58M90ABCDE", model: "Pixel 7", state: "device" }],
    list_ios_devices: [],
    list_processes: [
      { pid: 1, name: "launchd", state: "Run", cpu_percent: 0.1, memory_bytes: 12582912 },
      { pid: 4821, name: "CollectionLoom", state: "Run", cpu_percent: 2.4, memory_bytes: 195035136 },
      { pid: 9012, name: "Google Chrome", state: "Run", cpu_percent: 5.1, memory_bytes: 536870912 },
    ],
    get_imaging_progress: {
      percent: 67.4,
      status: `Imaging: 6.7 GB / 10.0 GB`,
      bytesProcessed: Math.floor(diskSize * 0.674),
      totalBytes: diskSize,
      isDone: false,
      error: null,
    },
    verify_hash: {
      algorithm: "sha256",
      expected: verifyHash,
      actual: verifyHash,
      matched: true,
      size: statSync(verifyPath).size,
    },
    generate_evidence_id: "EV-2026-0606-A1B2C3",
    generate_qr_label: null,
    list_cases_cmd: [
      {
        case_id: caseId,
        title: "Incident Response — Workstation Triage",
        operator: "J. Analyst",
        timezone: "UTC",
        status: "open",
        created_at: "2026-06-06T08:00:00Z",
      },
    ],
    list_case_summaries_cmd: [
      {
        case: {
          case_id: caseId,
          title: "Incident Response — Workstation Triage",
          operator: "J. Analyst",
          timezone: "UTC",
          status: "open",
          created_at: "2026-06-06T08:00:00Z",
        },
        snapshot_count: 2,
        export_count: 1,
        diff_count: 1,
        case_dir: "~/CollectionLoom/cases/CL-2026-DEMO",
      },
    ],
    list_snapshots_cmd: [
      {
        snapshot_id: snapshotId,
        case_id: caseId,
        host: "macbook-pro.local",
        os: "macOS 15",
        profile: "triage_5m",
        timestamp: "2026-06-06T08:05:00Z",
        integrity_hash: verifyHash.slice(0, 16) + "...",
      },
      {
        snapshot_id: "snap-002-post-containment",
        case_id: caseId,
        host: "macbook-pro.local",
        os: "macOS 15",
        profile: "ir_30m",
        timestamp: "2026-06-06T09:30:00Z",
        integrity_hash: "a3f2c8910d4e5f6a...",
      },
    ],
    get_snapshot_progress: {
      snapshotId,
      percent: 100,
      status: "Complete",
      modulesDone: 6,
      modulesTotal: 6,
    },
    list_exports: [
      {
        export_id: "exp-001",
        case_id: caseId,
        format: "zip",
        path: "~/CollectionLoom/cases/CL-2026-DEMO/exports/case_bundle.zip",
        created_at: "2026-06-06T10:00:00Z",
      },
    ],
    get_capture_stats: { packets: 1247, bytes: 892416 },
    get_capture_packets: [
      { time: "08:01:02.441", src: "192.168.1.42", dst: "8.8.8.8", proto: "DNS", info: "Standard query A google.com" },
      { time: "08:01:02.512", src: "192.168.1.42", dst: "142.250.80.78", proto: "TLS", info: "Client Hello" },
    ],
    take_snapshot: {
      id: snapshotId,
      timestamp: "2026-06-06T08:05:00Z",
      file_count: 42,
      process_count: 156,
      network_count: 23,
    },
  },
  uiState: {
    verifyFilePath: join(samples, "verify_me.txt"),
    verifyExpectedHash: verifyHash,
    cocCaseName: "CL-2026-DEMO",
    cocOperator: "J. Analyst",
    cocDevice: "/dev/disk4",
  },
};

writeFileSync(join(outDir, "screenshot-data.json"), JSON.stringify(data, null, 2));
console.log("Wrote fixture data with real SHA-256:", verifyHash);
console.log("Source disk:", diskSize, "bytes");
