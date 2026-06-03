# CollectionLoom 📀

[![Build](https://github.com/YSF-Studio/collectionloom/actions/workflows/build.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![Audit](https://github.com/YSF-Studio/collectionloom/actions/workflows/audit.yml/badge.svg)](https://github.com/YSF-Studio/collectionloom/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

> Portable forensic acquisition tool — evidence collection compliant with ISO 27037, built with **Tauri v2 + Rust + SvelteKit**.

## ✨ Features

| Feature | Details |
|---------|---------|
| **Disk Imaging** | Sector-by-sector acquisition (E01, dd, split-dd) with SHA-256 verification |
| **RAM Capture** | Volatile memory acquisition via avml / LiME |
| **Mobile Triage** | Android/iOS logical and physical acquisition |
| **Cloud Snapshot** | AWS/Azure/GCP evidence collection with temporary credentials |
| **Network Capture** | Packet capture via BPF with SPAN/mirror configuration |
| **System Snapshot** | Point-in-time file/process/network capture |
| **Acquire All** | Parallel multi-source acquisition with ETA tracking |
| **Write Blocker** | Hardware & software write protection |
| **Chain of Custody** | Evidence tracking with Ed25519 signatures |

## 🖥️ Screenshots

| Disk Imaging |
|:------------:|
| ![Disk](screenshots/collection_disk_imaging.png) |

> ℹ️ More screenshots coming soon — some features require the Tauri backend runtime.

## 🚀 Quick Start

```bash
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom/packages/collectionloom
npm install
npm run tauri dev
```

Or download the latest release from the [Releases](https://github.com/YSF-Studio/collectionloom/releases) page.

## 🏗️ Tech Stack

- **Backend:** Rust with Tauri v2
- **Frontend:** SvelteKit 5
- **Hashing:** SHA-256 via Rust `sha2` crate
- **Cloud:** AWS SDK, Azure SDK, GCP SDK via Rust
- **Capture:** BPF, avml integration
