# YSF Forensic Suite

[![CI/CD](https://github.com/YSF-Studio/ysf-forensic-suite/actions/workflows/ci.yml/badge.svg)](https://github.com/YSF-Studio/ysf-forensic-suite/actions/workflows/ci.yml)
[![Audit](https://github.com/YSF-Studio/ysf-forensic-suite/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/ysf-forensic-suite/actions/workflows/audit.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Three professional forensic applications built with **Tauri v2 + Rust + SvelteKit** — 100% offline, zero data collection.

## 🔗 Repositories

| App | Repo | CI Status | Purpose |
|-----|------|-----------|---------|
| 🟢 **ZipLoom** | [YSF-Studio/ziploom](https://github.com/YSF-Studio/ziploom) | [![Build](https://github.com/YSF-Studio/ziploom/actions/workflows/ci.yml/badge.svg)](https://github.com/YSF-Studio/ziploom/actions) | Archive inspection & threat detection |
| 🔴 **CollectionLoom** | [YSF-Studio/collectionloom](https://github.com/YSF-Studio/collectionloom) | [![Build](https://github.com/YSF-Studio/collectionloom/actions/workflows/ci.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions) | Portable forensic acquisition (ISO 27037) |
| 🔵 **AnalysisLoom** | [YSF-Studio/analysisloom](https://github.com/YSF-Studio/analysisloom) | [![Build](https://github.com/YSF-Studio/analysisloom/actions/workflows/ci.yml/badge.svg)](https://github.com/YSF-Studio/analysisloom/actions) | Forensic analysis workstation |

## ✨ Feature Highlights

### 📦 ZipLoom
- **Compress** — ZIP, TAR, TAR.GZ with drag-and-drop
- **Extract** — Multi-format extraction (ZIP, TAR, GZ, BZ2, XZ)
- **Inspect** — Preview archive contents without extracting, compression ratios, metadata
- **AES-256 Encryption** — PBKDF2 + AES-256-GCM for sensitive archives
- **100% Offline** — All processing runs locally

### 📀 CollectionLoom
- **Disk Imaging** — Bit-for-bit acquisition with SHA-256 verification
- **RAM Capture** — Memory acquisition (avml, winpmem support)
- **Mobile Triage** — Android & iOS logical acquisition
- **Cloud Snapshots** — AWS EBS, Azure Disk, GCP Persistent Disk
- **Network Capture** — Live packet acquisition with BPF filtering
- **Hash Verification** — Compare SHA-256/SHA-1/MD5 against expected values
- **Write Blocker** — Hardware & software write protection
- **Chain of Custody** — Evidence tracking with Ed25519 signatures
- **System Snapshot** — Point-in-time file/process/network capture

### 🔬 AnalysisLoom
- **NTFS/MFT Parser** — File browser with sorted deleted file recovery
- **File Preview** — Text, image, hex (interactive with byte select), archive
- **File Carving** — Multi-format signature-based recovery with progress
- **Timeline Analysis** — Chronological event correlation
- **Keyword Search** — Regex-based search across evidence
- **Case Management** — SQLite-based with evidence & findings tracking
- **Report Generation** — PDF & HTML reports with full audit trail
- **Bookmarks & Tags** — Mark files of interest with notes
- **Audit Trail** — ISO 27042-compliant action logging

## 🏗️ Architecture (Monorepo)

```
ysf-forensic-suite/
├── packages/
│   ├── core/           # Shared library: hashing, crypto, evidence, archive, ntfs, carving...
│   ├── ziploom/        # Archive compression & inspection
│   ├── collectionloom/ # Portable forensic acquisition
│   └── analysisloom/   # Forensic analysis workstation
└── .github/workflows/  # CI/CD across all apps
```

## 🧪 Test Suite

| Package | Tests | Status |
|---------|-------|--------|
| **ysf-core** | 17 unit + 24 comprehensive + 19 integration | ✅ All pass |
| **ZipLoom** | Rust + Frontend build | ✅ Clean |
| **CollectionLoom** | Rust + Frontend build | ✅ Clean |
| **AnalysisLoom** | Rust + Frontend build | ✅ Clean |

```bash
# Run all tests
cd packages/core && cargo test
```

## 🔒 Security

- **Gitleaks** — Secret scanning on every push
- **Cargo Audit** — Weekly dependency vulnerability scan
- **SBOM** — CycloneDX software bill of materials generated weekly
- **100% Offline** — Zero telemetry, zero data collection, no external network calls
- **AES-256-GCM** — Password-based encryption with PBKDF2 key derivation
- **Ed25519** — Chain of custody signing & verification

## 📄 License

MIT © YSF Studio — Built with ❤️ by Yusuf Shalahuddin
