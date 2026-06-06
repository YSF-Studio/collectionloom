# CollectionLoom User Guide

This guide describes how to use each acquisition module in CollectionLoom. All procedures follow **ISO/IEC 27037:2012** and **NIST SP 800-86** best practices. CollectionLoom runs **fully offline** — no evidence leaves your workstation.

---

## Before You Begin

1. **Prepare evidence storage** — Use NTFS, APFS, or ext4 with free space ≥ source drive size + 10%. Avoid FAT32 for images larger than 4 GB unless you enable split segments.
2. **Document the case** — Create a case in **System Snapshot** or **Case Dashboard** with operator name, timezone, and purpose.
3. **Enable write protection** — Connect a hardware write blocker when possible. Otherwise use **Enable Software Write-Blocker** before disk imaging.
4. **Record hashes** — Complete the **Chain of Custody** tab after each acquisition.

---

## Disk Imaging

**Tab:** Acquire Drive

1. Refresh the device list and select the source drive.
2. Enable software write-blocker if no hardware blocker is detected (titlebar badge turns green).
3. Choose format: **RAW** (`.dd`), **E01** (EnCase-compatible subset), or **AFF4** (ZIP container).
4. Set split size (e.g. 4096 MB) for large drives or FAT32 destinations. Leave at 0 for a single file on large-file filesystems.
5. Select destination path and click **Start Acquisition**.
6. Monitor progress (supports multi-TB sources — no application size cap).
7. Verify SHA-256 when acquisition completes.

**Large drives:** CollectionLoom uses block-device ioctls for accurate size detection on Linux, macOS, and Windows. Split segments are numbered `.00001`, `.00002`, etc.

---

## Write Blocker

CollectionLoom detects USB hardware blockers automatically. For software protection:

| Platform | Action | Requirement |
|----------|--------|-------------|
| Linux | BLKROSET read-only | Root or `sudo` |
| macOS | Force-unmount all volumes on disk | User confirmation |
| Windows | Disk read-only attribute | Administrator |

Use **Refresh** to re-check status before imaging.

---

## RAM Capture

**Tab:** RAM Capture

1. Confirm a capture tool is available (avml, LiME, DumpIt).
2. Ensure output volume has space ≥ RAM size.
3. Select tool and output path.
4. Run capture — do not sleep or hibernate the target during acquisition.
5. Hash the output file and record in chain of custody.

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

1. Create **read-only, time-limited** API credentials in your cloud console.
2. Enter provider (AWS / Azure / GCP), region, and resource identifier.
3. Initiate snapshot and wait for completion.
4. **Revoke credentials** immediately after download.

---

## Network Capture

**Tab:** Network Capture

1. Select interface (or configure SPAN/mirror upstream).
2. Optional: set BPF filter (e.g. `tcp port 443`).
3. Start capture; monitor packet count and size.
4. Stop capture and hash the `.pcapng` file.

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

1. Click **Detect Devices**.
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

1. Generate evidence ID and QR label.
2. Fill case name, operator, source device, and timezone.
3. Log each transfer action with timestamp.
4. Sign with Ed25519 and export PDF report.

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

---

## Support

Report issues: https://github.com/YSF-Studio/collectionloom/issues
