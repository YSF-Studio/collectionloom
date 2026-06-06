# CollectionLoom — PRD V1 (Final)

## 1. Product Summary

CollectionLoom is a **portable, cross-platform forensic collector** for capturing system state snapshots, comparing snapshots, and preparing structured handover packages for downstream analysis.

## 2. Problems Solved

- Forensic tool setup is too heavy during incident response.
- Artifact formats are inconsistent across endpoints and operating systems.
- Baseline vs. current-state comparison is slow and manual.
- Manual chain-of-custody is error-prone.
- Handoff to analysts and reporting teams duplicates effort.

## 3. Product Goals

1. Fast, consistent, repeatable capture.
2. Clear snapshot A/B compare (Added / Removed / Changed).
3. Tamper-evident artifacts (hash manifest + audit log).
4. Export-ready evidence packages for handoff.

## 4. Non-Goals (V1)

- Deep investigation canvas (entity graph, advanced timeline reasoning).
- Real-time multi-user collaboration.
- Built-in SIEM cloud ingestion.

## 5. Personas

- First responder (incident response)
- Digital forensic analyst
- SOC engineer needing portable triage

## 6. Critical Use Cases

1. Triage a suspected compromised endpoint (initial snapshot).
2. Re-snapshot after containment and compare deltas.
3. Package evidence + manifest for the investigation team.

## 7. Functional Requirements

### 7.1 Case Bootstrap

Create a case with minimal metadata: `case_id`, title, operator, purpose, timezone.

### 7.2 Capture Profiles

Presets: `triage_5m`, `ir_30m`, `deep_capture`. Custom profiles based on collector module lists.

### 7.3 Snapshot Runner

Run modular collectors across domains:

- **system** — hostname, OS, kernel, uptime, hardware
- **process** — process list, PPID, CPU/memory, command line
- **network** — connections, listening ports, DNS cache, ARP
- **autoruns/persistence** — startup items, services, scheduled tasks
- **users/sessions** — logged-in users, recent logons
- **selected logs** — event/system log excerpts

Per-module progress with partial success support. Per-collector timeouts to avoid blocking.

### 7.4 Snapshot Store

List and search snapshots. Metadata: `snapshot_id`, `case_id`, host, OS, timestamp, profile, collector version, integrity hash.

### 7.5 Compare Engine

Select Snapshot A and Snapshot B. Output categories:

- **added** — items only in Snapshot B
- **removed** — items only in Snapshot A
- **changed** — items in both with different values

Filter by domain (system/process/network/DLL) and severity heuristic. Each collector module defines its own comparison key.

### 7.6 Integrity & Custody

- Auto SHA-256 per artifact
- `hash_manifest.json` per snapshot
- `collector_audit.log` for per-module execution trace
- Signed hash chain (optional; V1 minimum is hash list)

### 7.7 Export

- **Normalized JSON pack** — all snapshot data in one versioned schema
- **Markdown summary report** — case, snapshot, and diff summary
- **ZIP bundle** — full evidence + manifest + custody log

## 8. Non-Functional Requirements

- **Portable-first** — run from a single folder (unzip → run)
- **Offline-capable** — no internet required
- **Cross-platform parity** — core features identical on Windows, macOS, Linux
- **Deterministic output schema** — versioned (`schema_version`) for backward compatibility
- **Low dependency** — Rust collector core, minimal external linking

## 9. Primary Field UX Flow

1. Open app → create or select case
2. Choose capture profile
3. Click **Start Snapshot**
4. View per-collector progress
5. (Optional) Compare with baseline snapshot
6. Review diff and filter by domain
7. Export evidence bundle

Target: **first snapshot in ≤ 3 clicks** after case selection.

## 10. Technical Architecture

- **Desktop UI:** Tauri + Svelte
- **Core engine:** Rust (collector module traits)
- **Storage:** file-based per case folder + optional lightweight SQLite index (V1 optional)
- **Package:** single binary + assets folder

## 11. Output Folder Structure

```
<case_folder>/
├── case.json
├── snapshots/
│   └── <snapshot_id>/
│       ├── snapshot_meta.json
│       ├── artifacts/
│       │   ├── system.json
│       │   ├── process.json
│       │   ├── network.json
│       │   └── ...
│       ├── hash_manifest.json
│       └── collector_audit.log
├── diffs/
│   └── <snapshotA>_vs_<snapshotB>.json
└── exports/
    ├── case_report.md
    ├── evidence_pack.json
    └── collection_bundle.zip
```

## 12. V1 KPIs

- Median triage snapshot time (`triage_5m` profile) < 5 minutes
- Total run failure rate < 5% (partial per-module success acceptable)
- 100% of snapshots include hash manifest
- Compare results readable by analysts without manual parsing

## 13. V1 Acceptance Criteria

- [x] App runs on macOS, Linux, and Windows
- [x] Create case, run snapshot, and compare A/B
- [x] Hash manifest + audit log always generated
- [x] Export JSON + Markdown report + ZIP bundle succeeds

## 14. V1.5 / V2 Backlog

- Rule-based noise reduction
- Signed bundle verification (Ed25519)
- Advanced severity heuristics
- Team reviewer workflow
- Collector plugin SDK
