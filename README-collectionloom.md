# CollectionLoom

[![Build](https://github.com/YSF-Studio/collectionloom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![Audit](https://github.com/YSF-Studio/collectionloom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

<p align="center">
  <img src="packages/collectionloom/public/icon.png" alt="CollectionLoom" width="120" />
</p>

> **Portable forensic acquisition toolkit** — evidence collection aligned with ISO/IEC 27037, built with **Tauri v2 + Rust + Svelte 5**. Runs fully offline on macOS, Windows, and Linux.

CollectionLoom helps first responders and forensic analysts capture disk images, volatile memory, network traffic, mobile backups, and system snapshots — then package evidence with hash manifests and chain-of-custody records for analyst handoff.

---

## Features

| Module | Description |
|--------|-------------|
| **Disk Imaging** | Sector-by-sector acquisition in RAW, native E01, or AFF4 format. SHA-256 verification (single- and multi-part), split images, HPA/DCO check, pre-imaging source integrity, acquisition summary |
| **Write Blocker** | Hardware auto-detect (Tableau/WiebeTech); software protection via titlebar or tab; disk picker in titlebar — no imaging tab required |
| **RAM Capture** | Volatile memory via avml / LiME / DumpIt with optional compression |
| **Mobile Triage** | Android ADB backup and iOS logical acquisition workflows |
| **Cloud Snapshot** | AWS EBS (Signature V4), Azure managed disk, and GCP persistent disk snapshots |
| **Network Capture** | Live packet capture with BPF filters and statistics |
| **System Snapshot** | Modular collectors (process, network, autoruns, users, logs) with triage / IR / deep profiles |
| **Compare Engine** | Snapshot A vs B diff — added, removed, and changed artifacts |
| **Acquire All** | Batch orchestration across disk, RAM, network, and mobile modules |
| **Encryption Scan** | Detect BitLocker, LUKS, VeraCrypt, FileVault, and encrypted containers |
| **Hash Verify** | SHA-256 integrity check against expected values |
| **Chain of Custody** | Ed25519-signed custody log with QR label PNG |
| **Case Dashboard** | Overview of cases, snapshots, exports, and diffs |
| **Export Bundle** | JSON pack, Markdown report, or ZIP bundle for analyst handoff |

---

## Screenshots

Screenshots are captured from the live UI in **light mode** using real sample files in [`samples/`](samples/) (SHA-256 hashes computed from actual bytes — not mocked).

Each capture exercises UI controls: disk imaging with acquisition summary (hash from `source_disk.img`), hash verify on `verify_me.txt`, write-blocker/HPA in titlebar, cloud credential picker, network capture stats, and chain-of-custody with evidence ID `BR2026-DSK-001`.

Regenerate: `npm run screenshots` from the repository root.

### Acquisition

| Disk Imaging | RAM Capture | Mobile Triage |
|:------------:|:-----------:|:-------------:|
| ![Disk Imaging](screenshots/collection_disk_imaging.png) | ![RAM Capture](screenshots/collection_ram_capture.png) | ![Mobile Triage](screenshots/collection_mobile_triage.png) |

| Cloud Snapshot | Network Capture | System Snapshot |
|:--------------:|:---------------:|:---------------:|
| ![Cloud](screenshots/collection_cloud_snapshot.png) | ![Network](screenshots/collection_network_capture.png) | ![Snapshot](screenshots/collection_system_snapshot.png) |

| Acquire All |
|:-----------:|
| ![Acquire All](screenshots/collection_acquire_all.png) |

### Analysis & Case Management

| Encryption Scan | Hash Verify | Case Dashboard |
|:---------------:|:-----------:|:--------------:|
| ![Encryption](screenshots/collection_encryption.png) | ![Verify](screenshots/collection_hash_verify.png) | ![Dashboard](screenshots/collection_case_dashboard.png) |

| Chain of Custody | Export Bundle | About |
|:----------------:|:-------------:|:-----:|
| ![CoC](screenshots/collection_chain_of_custody.png) | ![Export](screenshots/collection_export_bundle.png) | ![About](screenshots/collection_about.png) |

---

## Sample Files

The [`samples/`](samples/) directory contains real files for testing and documentation:

| File | Description |
|------|-------------|
| `verify_me.txt` | Hash verification target (SHA-256 in `expected.sha256`) |
| `expected.sha256` | Known-good SHA-256 for `verify_me.txt` |
| `source_disk.img` | 10 MB synthetic disk image for imaging tests |
| `case_notes.txt` | Sample case notes for export workflows |

Run integration tests against these files:

```bash
cd packages/collectionloom/src-tauri
cargo test forensic_test -- --nocapture
```

Regenerate documentation screenshots:

```bash
node scripts/prepare-screenshot-data.mjs
cd packages/collectionloom && VITE_FIXTURE_MODE=1 npm run build
node scripts/capture-screenshots.mjs
```

---

## Quick Start

### From source

```bash
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom
npm install
npm run tauri:dev
```

### Build (local only)

Pre-built binaries are **not published**. Build from source (repo root):

```bash
npm install
npm run tauri:build       # Tauri app + installers
npm run build:install   # same as tauri:build
npm run build:portable  # installers + portable zip in dist/portable/
```

See [docs/INSTALL.md](docs/INSTALL.md) for using each artifact after a local build.

---

## Documentation

| Document | Description |
|----------|-------------|
| [User Guide](docs/GUIDE.md) | Step-by-step acquisition procedures for every module |
| [Known Limitations](docs/LIMITATIONS.md) | Platform scope, verification boundaries, and operational caveats |
| [PRD V1](docs/PRD-EN.md) | Product requirements — snapshot, compare, export |
| In-app guides | Collapsible ISO 27037-aligned guides on each tab |

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2 |
| Backend | Rust (`ysf-core` shared library) |
| Frontend | Svelte 5 + Vite 6 |
| Imaging | Native E01 and AFF4 writers (no ewfacquire / aff4acquire) |
| Hashing | SHA-256, SHA-1, MD5 via Rust `sha2` / `md-5` |
| Signatures | Ed25519 via `ed25519-dalek` |
| Storage | Case folders under `~/CollectionLoom/cases/` |

---

## Write Blocker

| Mode | Platform | Method |
|------|----------|--------|
| Hardware | All | Auto-detect Tableau, WiebeTech, Logicube USB blockers |
| Software | Linux | BLKROSET ioctl — kernel read-only flag |
| Software | macOS | `diskutil unmountDisk force` then image via `/dev/rdiskN` |
| Software | Windows | `IOCTL_DISK_SET_DISK_ATTRIBUTES` read-only (Administrator) |

**Titlebar:** Select a disk from the dropdown, then click **Enable WB** — works without opening Disk Imaging or Acquire All. The titlebar badge shows **Write-Blocker Active** when hardware or software protection is confirmed.

See [Known Limitations](docs/LIMITATIONS.md) for platform caveats (software vs hardware, permissions, macOS behaviour).

---

## Known Limitations (summary)

CollectionLoom implements real HPA/DCO detection (Linux/Windows), AFF4 split sizing, multi-part hash verification, pre-imaging prefix integrity (sectors 0–99), network capture default 1 h timeout, Rust-side cloud credential files, imaging summaries, standard evidence IDs, and system theme — each with documented scope limits.

| Area | Key constraint |
|------|----------------|
| HPA/DCO | Not on macOS/NVMe; needs root/admin + direct block device |
| Source integrity | First 51,200 bytes only — not full-drive pre-hash |
| AFF4 split | Each part is a separate AFF4-L container |
| Network capture | Default 3600 s; `0` = infinite until manual stop |
| Cloud credentials | File-based via native picker — prepare JSON/INI beforehand |
| Imaging summary | `error_sectors` is 0 unless imaging aborts (fail-fast reads) |
| CI | No live hardware imaging or ATA pass-through in automated tests |

Full details: **[docs/LIMITATIONS.md](docs/LIMITATIONS.md)**

---

## License

MIT © [YSF Studio](https://github.com/YSF-Studio) — Yusuf Shalahuddin
