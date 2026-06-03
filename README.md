# YSF Forensic Suite 🔬

> **Justice shouldn't be paywalled.** — Open-source forensic tools built with Rust + Tauri.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

---

## 🧩 Suite Components

| App | Description | Status |
|-----|-------------|--------|
| **🗜️ [ZipLoom](https://github.com/YSF-Studio/ziploom)** | Archive compression, extraction & inspection — AES-256-GCM | ✅ Stable |
| **📀 [CollectionLoom](https://github.com/YSF-Studio/collectionloom)** | Portable forensic acquisition — ISO 27037 compliant | ✅ V2 Released |
| **🔬 [AnalysisLoom](https://github.com/YSF-Studio/analysisloom)** | Forensic analysis workstation — NTFS, carving, timeline | 🚧 In Development |

---

## 📀 CollectionLoom V2 — What's New

### 6 Acquisition Modules + 2 Advanced Modes

| Module | Features |
|--------|----------|
| **💿 Disk Imaging** | DD/RAW, split, verify, HPA/DCO detect, SSD TRIM warning, ETA |
| **🧠 RAM Capture** | WinPmem/LiME/MRS, auto hash, process list |
| **📱 Mobile Triage** | Android ADB + iOS iTunes backup, Faraday reminder |
| **☁️ Cloud Snapshot** | AWS, Azure, GCP, Alibaba — REST API (no SDK) |
| **🌐 Network Capture** | BPF filter, packet preview, ring buffer, auto hash |
| **🛡️ Write Blocker** | macOS/Win/Linux, status indicator |
| **⭐ Acquire All** | Parallel mode — satu klik, semua modul berjalan |
| **📸 Snapshot** | Baseline → Post-Execution → Delta Analysis |

### Chain of Custody
- Evidence ID: `CL-YYYYMMDD-XXXX`
- Ed25519 digital signing
- QR code label
- PDF report

### Guided Mode
Setiap modul dilengkapi panduan langkah demi langkah berbasis **ISO 27037** dan **NIST SP 800-86**.

---

## 🖥️ Screenshot

![CollectionLoom V2](https://raw.githubusercontent.com/YSF-Studio/collectionloom/main/screenshots/collection_disk_imaging.png)
*Disk Imaging tab — HPA/DCO detection, progress with ETA, GuideCard*

---

## 🚀 Quick Start

```bash
# Clone the suite
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom/packages/collectionloom

# Install & build
npm install
npm run tauri build
```

---

## 🏗️ Architecture

```
ysf-forensic-suite/
├── packages/
│   ├── ziploom/         # Archive utility (Tauri + SvelteKit)
│   ├── collectionloom/  # Acquisition toolkit (Tauri + SvelteKit)
│   ├── analysisloom/    # Analysis workstation (Tauri + SvelteKit)
│   └── core/            # Shared Rust engine
└── README.md
```

### Shared Rust Core (`packages/core/`)
- `hashing` — SHA-256, SHA-1, MD5, entropy
- `crypto` — Ed25519 signing, AES-256-GCM
- `imaging` — Disk imaging engine
- `snapshot` — System snapshot + delta analysis
- `network` — libpcap capture engine
- `mobile` — ADB + iOS backup
- `cloud` — AWS, Azure, GCP, Alibaba
- `report` — PDF generation
- `evidence` — Chain of Custody model
- `ntfs`, `carving`, `preview` — Analysis modules

---

## 📄 License

MIT © [YSF Studio](https://github.com/YSF-Studio) — Built with ❤️ by Yusuf Shalahuddin
