# Known Limitations and Scope

This document describes what CollectionLoom **does not** guarantee, platform-specific constraints, and operational boundaries. Read this before relying on outputs in contested or legal proceedings.

---

## General

| Topic | Limitation |
|-------|------------|
| **Legal admissibility** | CollectionLoom assists acquisition workflows aligned with ISO/IEC 27037; it does not replace organizational policy, qualified examiner review, or independent hash verification. |
| **Hardware validation** | Automated tests run in CI without physical evidence drives, write blockers, or live network taps. Field behaviour must be validated on your target OS and hardware. |
| **Preview mode** | Running the UI in the browser (`VITE_FIXTURE_MODE=1` or without Tauri) uses fixture data — not real devices or imaging. |
| **Administrator / root** | Disk imaging, software write-blocking, HPA/DCO checks, and RAM capture often require elevated privileges on the acquisition workstation. |

---

## Write Blocker

| Topic | Limitation |
|-------|------------|
| **Software vs hardware** | Software write-blocking reduces risk but is **not** a substitute for certified hardware blockers on contested evidence. |
| **macOS** | Protection is achieved by force-unmounting volumes and imaging via the raw disk path (`/dev/rdiskN`). There is no kernel-level BLKROSET equivalent. |
| **Windows** | Software read-only requires **Administrator**; some USB bridges may not honour IOCTL read-only. |
| **Linux** | BLKROSET requires **root** or sufficient capability on the block device node. |
| **Titlebar controls** | The titlebar disk picker and **Enable WB** button operate independently of the Disk Imaging tab. You must still select the correct physical device — there is no automatic target validation. |
| **Session scope** | Software protection state is tracked per session; reconnecting or re-enumerating a drive may require re-enabling protection. |

---

## HPA / DCO Detection

| Topic | Limitation |
|-------|------------|
| **Supported platforms** | Real ATA pass-through detection runs on **Linux** (`HDIO_DRIVE_CMD`) and **Windows** (`IOCTL_ATA_PASS_THROUGH_DIRECT`). |
| **macOS** | Returns an explicit **unsupported** report — most Mac storage paths do not expose ATA pass-through for attached drives. |
| **NVMe** | HPA/DCO ATA checks are **not applicable**; NVMe uses a different command set. |
| **Requirements** | Direct block-device access and elevated privileges are required; detection over USB bridges depends on bridge firmware forwarding ATA commands. |
| **Scope** | Compares IDENTIFY max LBA vs native max (HPA) and DCO identify max (DCO). False negatives are possible if firmware or drivers block pass-through. |

---

## Disk Imaging

| Topic | Limitation |
|-------|------------|
| **E01 writer** | Native E01 subset — not full EnCase feature parity (compression levels, case metadata, etc.). |
| **AFF4 split** | Each split part is a **separate AFF4-L ZIP container** (`{stem}.00001.aff4`, …), not a single multi-segment AFF4 stream. |
| **FAT32 destinations** | Use split size (e.g. 4096 MB) for images larger than 4 GB; otherwise the write will fail on FAT32. |
| **SSD / TRIM** | Deleted data on SSDs may be unrecoverable before acquisition; the UI warns but cannot prevent TRIM at the OS/firmware level. |
| **Encrypted volumes** | BitLocker, FileVault, LUKS, etc. are detected where possible; imaging captures ciphertext unless decrypted upstream. |

---

## Hash Verification

| Topic | Limitation |
|-------|------------|
| **Single-part RAW** | Full-file SHA-256 is computed during imaging and re-hashed after write when verify is enabled. |
| **Multi-part RAW** | Verification concatenates part file bytes in order and compares to the stream hash from acquisition. |
| **Multi-part AFF4** | Verification hashes the **data stream** extracted from each part container, not the raw ZIP bytes. |
| **Split E01** | Follow RAW-style part naming where supported; verify behaviour matches the selected format path in `imaging_format.rs`. |
| **Performance** | Post-write verification re-reads all output data — large multi-TB jobs add significant time. |

---

## Pre-Imaging Source Integrity

| Topic | Limitation |
|-------|------------|
| **Scope** | Hashes the first **51,200 bytes** (sectors 0–99, 512 bytes/sector) before imaging starts. |
| **Post-check** | Re-reads the same prefix from the source after imaging and compares the image prefix to the pre-hash. |
| **Not full-drive** | Does **not** hash the entire source before acquisition — only a fixed prefix for tamper detection during the session. |
| **Live systems** | On a running OS disk, the prefix may change legitimately (boot activity); interpret failures in context. |

---

## Imaging Summary Report

| Topic | Limitation |
|-------|------------|
| **Fields** | Reports sectors read, duration, average speed, SHA-256, source integrity, and bad-sector count. |
| **Bad sectors** | Unreadable sectors are **zero-filled** and logged to `{image}.bad_sectors.log`; acquisition continues (does not fail). |
| **CoC integration** | Hashes are also recorded in chain-of-custody actions; the summary card is an in-app convenience, not a signed report. |
| **Acquisition audit** | Disk/RAM acquisitions append to `~/CollectionLoom/cases/acquisition_audit.jsonl` and `{case}/logs/acquisition_audit.jsonl` when linked to a case. |

---

## Network Capture

| Topic | Limitation |
|-------|------------|
| **Default timeout** | **3600 seconds (1 hour)** when `max_duration_secs` is omitted. Setting **0** means **infinite** capture until manual stop — the UI shows a warning. |
| **Permissions** | Packet capture requires appropriate OS permissions (`libpcap`, admin on some platforms). |
| **Storage** | Long captures on busy interfaces can fill disk; monitor output path free space. |
| **Encrypted traffic** | Captured TLS payload is ciphertext unless keys are available elsewhere. |

---

## Cloud Snapshot

| Topic | Limitation |
|-------|------------|
| **Credentials** | API keys are loaded via a **native file picker** in Rust — not typed into the web UI. You must prepare a credential file beforehand. |
| **AWS** | Uses EC2 Query API with **Signature Version 4**; credential file format must match what `cloud.rs` expects (JSON keys). |
| **Azure / GCP** | Provider-specific token or service-account JSON paths; there is no multi-step credential wizard in-app. |
| **Operational security** | Revoke cloud credentials immediately after snapshot download; CollectionLoom does not store secrets in the frontend. |
| **Offline core** | Cloud module requires network access to the provider API at snapshot time. |

---

## Chain of Custody & Evidence IDs

| Topic | Limitation |
|-------|------------|
| **ID format** | `[CASE-INITIALS]-[MEDIA-TYPE]-[SEQUENCE]` e.g. `BR2026-DSK-001`. Sequence counters persist under `~/.ysf/evidence_*.counter`. |
| **Case initials** | Derived from the case name (first two letters + year) when generated via chain-of-custody — not a court-assigned case number unless you enter one. |
| **Preview fixtures** | Browser preview mode may still show legacy example IDs in screenshot fixtures. |
| **Signatures** | Ed25519 signs custody records; key management and trust anchors are the operator's responsibility. |

---

## Theme & UI

| Topic | Limitation |
|-------|------------|
| **System theme** | Follows `prefers-color-scheme` when set to **System**; changes apply on cycle or when the OS theme changes (listener active). |
| **Accessibility** | Some form labels in the UI have known a11y warnings in the Svelte build — does not affect acquisition logic. |

---

## Testing & CI

| Topic | Limitation |
|-------|------------|
| **CI matrix** | GitHub Actions builds and runs unit/integration tests on ubuntu/macos/windows; **no live disk imaging or HPA/DCO pass-through** in CI. |
| **Sample data** | Integration tests use synthetic files in [`samples/`](../samples/) (e.g. 10 MB `source_disk.img`). |
| **Regression** | Always verify critical workflows on your acquisition workstation before field deployment. |

---

## Related Documentation

- [User Guide](GUIDE.md) — step-by-step procedures
- [README](../README.md) — features and quick start
- [PRD V1](PRD-EN.md) — product requirements

Report gaps or incorrect behaviour: https://github.com/YSF-Studio/collectionloom/issues
