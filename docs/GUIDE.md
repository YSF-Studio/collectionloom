# CollectionLoom User Guide

This guide describes how to use each acquisition module in CollectionLoom. All procedures follow **ISO/IEC 27037:2012** and **NIST SP 800-86** best practices. CollectionLoom runs **fully offline** — no evidence leaves your workstation.

---

## Before You Begin

1. **Prepare evidence storage** — Use NTFS, APFS, or ext4 with free space ≥ source drive size + 10%. Avoid FAT32 for images larger than 4 GB unless you enable split segments.
2. **Document the case** — Create a case in **System Snapshot** or **Case Dashboard** with operator name, timezone, and purpose.
3. **Enable write protection** — Connect a hardware write blocker when possible. From the **titlebar**, select the target disk and click **Enable WB** (no need to open Disk Imaging first). Alternatively use **Enable Software Write-Blocker** on the Disk Imaging tab.
4. **Record hashes** — Complete the **Chain of Custody** tab after each acquisition.
5. **Run pre-flight check** — Open **Prerequisites** (Case Info) to verify external tools (RAM, mobile, network) and privilege level before starting acquisition.

See [Known Limitations](LIMITATIONS.md) for platform scope and verification boundaries.

---

## Prerequisites (External Tools)

CollectionLoom ships core acquisition in **pure Rust** (disk imaging, hashing, CoC signing, PDF, carving). Some modules depend on **system libraries** or **external binaries** that must be installed separately:

| Module | Dependency | Install |
|--------|------------|---------|
| **Network Capture** | libpcap (Unix) or Npcap (Windows) | Linux: `libpcap-dev` · macOS: Xcode CLT · Windows: [Npcap](https://npcap.com) |
| **RAM (Linux)** | avml or LiME | `cargo install avml` or build [LiME](https://github.com/504ensicsLabs/LiME) |
| **RAM (Windows)** | WinPmem | [Velocidex/WinPmem](https://github.com/Velocidex/WinPmem) |
| **macOS volatile triage** | sysdiagnose / process dump / network state | Use the Apple Volatile Data section in the RAM guide |
| **Mobile Android** | adb | [Android Platform Tools](https://developer.android.com/tools/releases/platform-tools) |
| **Mobile iOS** | libimobiledevice | macOS: `brew install libimobiledevice` · Linux: `libimobiledevice-utils` |
| **Cloud Snapshot** | Internet HTTPS | Outbound access to AWS/Azure/GCP APIs |
| **Write blocker / HPA** | root or Administrator | Linux: `sudo` · Windows: Run as Administrator |

Use **Prerequisites** in the sidebar to detect what is installed on the current workstation. Missing tools show install hints; the app warns at startup when required binaries are absent.

---

## Portable Forensic Kit (Field / IR)

For incident response on suspect machines, **do not install** packages on the evidence system. Prepare a forensic USB with the full kit:

```
CollectionLoom/
├── CollectionLoom.app   (or collectionloom / CollectionLoom.exe)
├── tools/               avml, adb, idevice_id, … (static binaries)
├── cases/               evidence output
└── tools/manifest.json  SHA-256 hashes (recommended)
```

1. **Boot from write-blocked USB** when possible — do not run from the suspect internal drive.
2. **Launch CollectionLoom** from the USB — zero install on target.
3. **Open Prerequisites** — confirm `./tools/` binaries are found and hash-verified.
4. **Acquire All** — output to `cases/` on destination USB, not the source volume.
5. **Remove USB** when done — no files left on suspect machine (except documented artifacts).

CollectionLoom resolves `./tools/` **before** system PATH. See `tools/README.txt` in the repository for layout and hash generation.

Set `COLLECTIONLOOM_KIT_ROOT` to override kit root in custom deployments.

---

## Disk Imaging

**Tab:** Acquire Drive

1. Refresh the device list and select the source drive.
2. Enable software write-blocker if no hardware blocker is detected (titlebar **Enable WB** or tab button; badge turns green).
3. Optional: click **Check HPA/DCO** to scan for hidden ATA areas (Linux/Windows + root/admin; not supported on macOS/NVMe — see [Limitations](LIMITATIONS.md)).
4. Choose format: **RAW** (`.dd`), **E01** (EnCase-compatible subset), or **AFF4** (ZIP container).
5. Set split size (e.g. 4096 MB) for large drives or FAT32 destinations. Leave at 0 for a single file on large-file filesystems. AFF4 split produces separate `{name}.00001.aff4` containers per part.
6. Select destination path and enable **Verify after write** for SHA-256 check (single- and multi-part).
7. Click **Start Acquisition** and monitor progress (supports multi-TB sources).
8. Review the **Acquisition Summary** card (sectors, duration, speed, source integrity, SHA-256) when complete.

**Source integrity:** Before imaging, the first 51,200 bytes (sectors 0–99) are hashed and compared after acquisition. This detects tampering during the session but is **not** a full-drive pre-hash.

**Large drives:** CollectionLoom uses block-device ioctls for accurate size detection on Linux, macOS, and Windows. Split segments are numbered `.00001`, `.00002`, etc.

---

## Write Blocker

CollectionLoom detects USB hardware blockers automatically. For software protection:

**Titlebar (recommended):**

1. Open the disk dropdown in the titlebar (list loads at startup; click **↻** to refresh).
2. Select the evidence drive.
3. Click **Enable WB**. Status appears in the titlebar badge.

This works from any tab — you do not need to open Disk Imaging or Acquire All first.

**Disk Imaging / Acquire All tabs** also expose Enable, Disable, and Refresh for the selected disk.

| Platform | Action | Requirement |
|----------|--------|-------------|
| Linux | BLKROSET read-only | Root or `sudo` |
| macOS | Force-unmount all volumes on disk | User confirmation; image via raw path |
| Windows | Disk read-only attribute | Administrator |

Use **Refresh** to re-check status before imaging. Software blocking is not a substitute for certified hardware on contested evidence — see [Limitations](LIMITATIONS.md).

---

## RAM Capture

**Tab:** RAM Capture

CollectionLoom uses two RAM acquisition modes:

- **Mode 1 (Recommended)** - automatic, platform-aware selection that prefers the safest practical path on the current OS.
- **Mode 2 (Advanced)** - manual tool selection for investigators who need a specific RAM workflow.

1. Confirm that the selected mode matches the target platform.
2. Ensure the output volume has space greater than or equal to the RAM size.
3. In Mode 2, select the specific tool and output path.
4. Run capture. Do not sleep or hibernate the target during acquisition.
5. Hash the output file and record it in chain of custody.

**macOS note:** CollectionLoom does not provide a raw RAM dump workflow on macOS. Use the app for the supported acquisition paths on that platform, but plan on disk, triage, and artifact collection rather than universal live memory dumping.

**Volatility note:** Volatility is an analysis framework for memory images, not a memory acquisition tool. You still need a separate acquisition step before Volatility can analyze the dump.

## Apple Volatile Data

**Platform:** macOS (Intel and Apple Silicon)

CollectionLoom does not attempt raw RAM dumping on macOS. Instead, use this section for alternative volatile sources:

1. Capture a process list and active process metadata.
2. Collect network state and open connections.
3. Gather login/session, autorun, and user activity artifacts where available.
4. Capture system diagnostics or live-response artifacts that help preserve volatility context.
5. Hash and record every output in chain of custody.

This is intentionally an alternative-source workflow, not a raw memory dump workflow. Treat it as triage and live response support for macOS.

---

## Mobile Triage

**Tab:** Mobile Triage

1. Isolate the device (airplane mode / Faraday bag).
2. Connect via USB and refresh device list.
3. Run ADB backup (Android) or logical backup (iOS) to the case folder.
4. Hash backup files immediately.

---

## Cloud Snapshot

**Tab:** Cloud Snapshot

1. Create **read-only, time-limited** API credentials in your cloud console and save them to a local file (JSON or INI — provider-specific).
2. Click **Choose credentials file** — the native file picker loads secrets in Rust; credentials are **not** entered in the web UI.
3. Select provider (AWS / Azure / GCP), region, and resource identifier.
   - **AWS:** EC2 Query API with **Signature Version 4** (access key + secret key in credential file).
   - **Azure / GCP:** bearer token or service-account JSON path as documented for each provider.
4. Initiate snapshot and wait for completion.
5. **Revoke credentials** immediately after download.

See [Limitations](LIMITATIONS.md#cloud-snapshot) for credential format and scope.

---

## Network Capture

**Tab:** Network Capture

1. Select interface (or configure SPAN/mirror upstream).
2. Optional: set BPF filter (e.g. `tcp port 443`).
3. Set **Max duration (seconds)** — default **3600** (1 hour). Use **0** only if you intend to stop capture manually (infinite; UI shows a warning).
4. Start capture; monitor packet count and size.
5. Stop capture (or wait for timeout) and hash the `.pcapng` file.

---

## System Snapshot

**Tab:** System Snapshot

1. Create or select a case.
2. Choose profile: **Triage 5m**, **IR 30m**, or **Deep Capture**.
3. Run snapshot — collectors gather system, process, network, autorun, user, and log artifacts.
4. Compare two snapshots to see added, removed, and changed items.

---

## Acquire All

**Tab:** Acquire All

1. Click **Detect Sources**.
2. Enable modules (Disk, RAM, Network, Mobile).
3. Configure split size for disk module if needed.
4. Enable write-blocker on selected disk (automatic before imaging if inactive).
5. Click **Start Acquire All** — modules run in sequence.

See the in-app **Acquire All Guide** for detailed steps.

---

## Encryption Detection

**Tab:** Encryption

Run a scan to identify BitLocker, LUKS, VeraCrypt, FileVault, and encrypted containers. Document encrypted volumes in your case report before imaging.

---

## Hash Verification

**Tab:** Hash Verify

1. Select algorithm (SHA-256 recommended).
2. Point to evidence file (e.g. `samples/verify_me.txt`).
3. Enter expected hash from chain of custody.
4. Run verification — match confirms integrity.

---

## Chain of Custody

**Tab:** Custody Chain

1. Generate evidence ID — format `[CASE-INITIALS]-[MEDIA-TYPE]-[SEQUENCE]` (e.g. `BR2026-DSK-001`). Sequence counters persist per case/media under `~/.ysf/`.
2. Generate QR label PNG.
3. Fill case name, operator, source device, and timezone.
4. Log each transfer action with timestamp.
5. Sign with Ed25519 and export PDF report.

---

## Export & Handoff

**Tab:** Export Bundle

Export formats:

| Format | Contents |
|--------|----------|
| JSON Pack | Normalized `evidence_pack.json` with schema version |
| Markdown | Human-readable case report |
| ZIP Bundle | Full case folder archive with manifest |

Use **Export Bundle** from Case Dashboard to package evidence for analyst handoff.

---

## Case Dashboard

Overview of all cases with snapshot, export, and diff counts. Open case folders directly from the UI.

---

## References

- ISO/IEC 27037:2012 — Digital evidence identification, collection, acquisition, and preservation
- NIST SP 800-86 — Integrating forensic techniques into incident response
- NIST SP 800-101 Rev. 1 — Mobile device forensics
- NIST CFReDS — https://cfreds.nist.gov
- [Known Limitations](LIMITATIONS.md) — platform scope and verification boundaries

---

## Support

Report issues: https://github.com/YSF-Studio/collectionloom/issues
