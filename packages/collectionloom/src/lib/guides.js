/**
 * guides.js — Step-by-step forensic acquisition guides for CollectionLoom.
 *
 * Each guide follows ISO 27037 best practices and NIST SP 800-86 guidelines.
 * Used by GuideCard.svelte to render collapsible instruction panels.
 *
 * @typedef {Object} GuideStep
 * @property {string} title       — Short action heading
 * @property {string} description — Detailed instructions
 * @property {string} [warning]   — ⚠️ Caution (optional)
 *
 * @typedef {Object} Guide
 * @property {string}  title       — Panel heading
 * @property {string}  icon        — Emoji / icon identifier
 * @property {GuideStep[]} steps   — Ordered procedure
 * @property {string[]} references — Citations / resources
 */

/** @type {Guide} */
export const diskImagingGuide = {
  title: "Disk Imaging Guide",
  icon: "●",   // disk
  steps: [
    {
      title: "Verify write blocker",
      description:
        "Use a hardware write blocker when possible (Tableau, WiebeTech, etc.) — CollectionLoom auto-detects USB blockers and shows a green badge in the titlebar. If no hardware blocker is available, select the source disk and click **Enable Software Write-Blocker**: Linux sets BLKROSET read-only; macOS force-unmounts volumes before imaging via `/dev/rdiskN`; Windows sets the disk read-only via IOCTL (Administrator required).",
      warning:
        "Software write-blocking reduces risk but is not a substitute for certified hardware on contested evidence. Never mount suspect volumes read-write before imaging.",
    },
    {
      title: "Select source and destination",
      description:
        "Identify the source device from the system's device list. Choose a destination path on a dedicated evidence storage volume with enough free space (source size + 10% for split/verification overhead). Format destination as NTFS or ext4, never FAT32 (4 GB file limit).",
    },
    {
      title: "Configure split and verification options",
      description:
        "For drives larger than 4 GB, set split size (e.g., 4096 MB) when the destination volume uses FAT32 or when you need manageable chunk files. CollectionLoom supports u64 byte counts — there is no built-in size cap. Enable verification so SHA-256 is computed during imaging and compared after write (single-part images).",
    },
    {
      title: "Acquire the image",
      description:
        "Launch acquisition. The tool reads the source device sector-by-sector and writes an E01 (EnCase) or dd / split-dd format to the destination. Monitor real-time progress including bytes copied, percentage complete, and estimated time remaining.",
    },
    {
      title: "Verify hash and document chain of custody",
      description:
        "After acquisition completes, compare the computed hash (SHA-256) of the source against the output image. Log the algorithm and hash value in the chain-of-custody (CoC) form. Store the image on write-once media or a tamper-evident NAS share.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Guidelines for identification, collection, acquisition and preservation of digital evidence",
    "NIST SP 800-86 — Guide to Integrating Forensic Techniques into Incident Response",
    "NIST CFReDS — Computer Forensic Reference Data Sets (https://cfreds.nist.gov)",
  ],
};

/** @type {Guide} */
export const ramCaptureGuide = {
  title: "RAM Capture Guide",
  icon: "◇",   // ram
  steps: [
    {
      title: "Check available tools",
      description:
        "Verify that a memory acquisition tool (e.g., LiME, avml, winpmem, or DumpIt) is present on the target system. For Linux, ensure LiME kernel module or avml static binary is staged. Confirm sufficient free space on the output volume (at least RAM size + 256 MB).",
      warning:
        "Running an untrusted binary on a suspect machine may alter evidence — use a trusted, hashed acquisition utility from read-only media.",
    },
    {
      title: "Close unnecessary applications",
      description:
        "Minimize active processes on the target to reduce RAM churn during capture. Do not power off or reboot the system — volatile data is lost on shutdown. Avoid running disk-intensive operations alongside capture.",
    },
    {
      title: "Capture volatile memory",
      description:
        "Execute the acquisition tool with appropriate parameters. For Linux with LiME: `insmod lime.ko path=/evidence/ram.lime format=lime`. For avml: `./avml /evidence/ram.avml`. For Windows: run DumpIt or winpmem and specify the output path. Wait for completion — do not interrupt.",
      warning:
        "Some anti-virus / EDR software may flag memory acquisition tools as malicious. Pre-authorise the toolpath or temporarily pause endpoint protection if safe to do so.",
    },
    {
      title: "Compute and record hash",
      description:
        "Generate a SHA-256 hash of the resulting memory dump file immediately after capture. Record the hash alongside the acquisition timestamp, tool version, and target hostname in the CoC documentation. Store the dump on encrypted, access-controlled media.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Digital evidence acquisition procedures",
    "NIST SP 800-86 — Live response and volatile data collection",
    "RFC 3227 — Guidelines for Evidence Collection and Archiving",
  ],
};

/** @type {Guide} */
export const mobileTriageGuide = {
  title: "Mobile Triage Guide",
  icon: "☎",   // mobile
  steps: [
    {
      title: "Isolate device from network",
      description:
        "Place the device in a Faraday bag or enable Airplane Mode immediately to prevent remote wipe commands, incoming messages, or cloud sync from altering evidence. If the device is locked, do not attempt to unlock it by guessing PINs (may trigger wipe).",
      warning:
        "A device left connected to cellular/Wi-Fi can be remotely wiped in seconds. Use a shielded container for transport and storage.",
    },
    {
      title: "Enable USB debugging / developer mode",
      description:
        "On Android: boot into recovery mode or use a trusted MFD (Mobile Forensic Device) to enable ADB debugging without touching the screen. On iOS: place the device into DFU mode or use a checkpoint-compatible bootloader exploit for logical acquisition. Document every interaction.",
    },
    {
      title: "Acquire logical backup",
      description:
        "Run the acquisition tool to create a logical backup of user data. For Android via ADB: `adb backup -apk -shared -all -system -f backup.ab`. For advanced extraction, use tools like AFLogical OSE or commercial suites. Collect call logs, SMS, contacts, installed apps, and media files.",
    },
    {
      title: "Hash and secure the backup",
      description:
        "Compute SHA-256 of all acquired files and the container file (`.ab` or `.zip`). Record the hashes in the CoC. Store on encrypted media. If the device supports file-level encryption, also capture the encryption metadata for later analysis.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Mobile device acquisition considerations",
    "NIST SP 800-86 — Cell phone and PDA forensic procedures",
    "NIST IR 800-101 Rev 1 — Guidelines on Mobile Device Forensics",
  ],
};

/** @type {Guide} */
export const cloudEvidenceGuide = {
  title: "Cloud Evidence Guide",
  icon: "☁",   // cloud
  steps: [
    {
      title: "Generate temporary API credentials",
      description:
        "Log into the cloud provider's IAM console and create a time-limited API key pair with read-only permissions. Set expiration to the minimum viable window (e.g., 2 hours). Scope the policy to only the target resources needed — never use full-admin keys.",
      warning:
        "API keys with excessive permissions or no expiration create a post-exercise risk. Always scope to least-privilege and set a revocation timer.",
    },
    {
      title: "Snapshot target resources",
      description:
        "Initiate snapshots of virtual machine disks (EBS volumes, Azure managed disks, GCP persistent disks) and database instances. Tag each snapshot with the case ID, timestamp, and operator name. Wait for snapshots to reach 'completed' status before proceeding.",
    },
    {
      title: "Download configuration and logs",
      description:
        "Use the cloud provider's CLI or API to export configuration data: IAM policies, network ACLs, VPC flow logs, CloudTrail / Activity Log events, and instance metadata. Save as structured data (JSON/CSV) with timestamps. Capture at least 90 days of logs when available.",
    },
    {
      title: "Revoke temporary credentials",
      description:
        "Immediately revoke the temporary API keys after all data has been collected. Verify the keys are disabled in the IAM console. Log the revocation action in the CoC form with the key ID and revocation timestamp.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Remote / cloud evidence collection",
    "NIST SP 800-86 — Virtual and cloud forensic considerations",
    "CSA (Cloud Security Alliance) — Mappings to NIST SP 800-86",
  ],
};

/** @type {Guide} */
export const networkCaptureGuide = {
  title: "Network Capture Guide",
  icon: "⊙",   // network
  steps: [
    {
      title: "Configure SPAN / port mirroring",
      description:
        "If capturing passively, configure a SPAN or port mirror on the managed switch to duplicate traffic from the target VLAN/port to the capture interface. Ensure the capture NIC is in promiscuous mode. For inline capture (honeypot), position the capture device between the target and the gateway.",
      warning:
        "A poorly provisioned SPAN port can drop packets under high load. Ensure the switch CPU and capture storage can handle the expected bandwidth. Test with a known traffic pattern before beginning.",
    },
    {
      title: "Set capture filter (BPF)",
      description:
        "Define a Berkeley Packet Filter (BPF) to limit captured traffic to relevant protocols — e.g., `tcp` or `udp port 53` or `host 10.0.0.1`. This reduces noise and storage requirements. Use a capture length (snaplen) of at least 65535 bytes to avoid truncating packets.",
    },
    {
      title: "Begin packet capture",
      description:
        "Launch tcpdump, tshark, or Wireshark with the configured filter and output file. For tcpdump: `tcpdump -i eth0 -s 65535 -w evidence.pcap -C 1024 -W 10 'tcp port 80 or tcp port 443'`. The `-C` and `-W` flags rotate files every 1024 MB, keeping the last 10.",
    },
    {
      title: "Verify and hash capture files",
      description:
        "After capture completes, validate the pcap files by opening them in Wireshark or using `capinfos evidence.pcap`. Generate SHA-256 hashes for each pcap file and record alongside start/end timestamps and total packet count in the CoC.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Network evidence acquisition",
    "NIST SP 800-86 — Network-based evidence",
    "NIST SP 800-94 — Guide to Intrusion Detection and Prevention Systems",
  ],
};

/** @type {Guide} */
export const writeBlockerGuide = {
  title: "Write Blocker Guide",
  icon: "⚷",   // password/encrypt
  steps: [
    {
      title: "Inspect the write blocker hardware",
      description:
        "Visually inspect the write blocker device for physical damage. Verify the correct interface (SATA, IDE, USB Bridge, NVMe) matches the source drive. Ensure the device's firmware is up-to-date and documented. Connect the write blocker to the acquisition workstation via USB or eSATA.",
      warning:
        "Cheap or counterfeit write blockers may not enforce read-only at the hardware level. Use only devices listed on NIST's approved tools list or with a published hardware design.",
    },
    {
      title: "Connect and enable the device",
      description:
        "Connect the source drive to the write blocker, then power on the blocker before connecting to the workstation. Confirm the LED indicator shows 'Protected' / 'Read-Only'. CollectionLoom auto-detects Tableau/WiebeTech USB blockers and shows a green badge in the titlebar. Without hardware: click **Enable Software Write-Blocker** — Linux: BLKROSET (`/sys/block/<dev>/ro` = 1); macOS: `diskutil unmountDisk force` then image via `/dev/rdiskN`; Windows: IOCTL read-only (run as Administrator).",
    },
    {
      title: "Verify read-only state before imaging",
      description:
        "Attempt to write a test marker to the device: `dd if=/dev/zero of=/dev/sdX bs=512 count=1` should fail with 'Read-only file system' or 'Permission denied'. If the write succeeds, abort immediately and replace the blocker. Never image a drive connected through a failed blocker.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Hardware write-blocking requirements",
    "NIST CFTT — Computer Forensics Tool Testing Program (https://www.nist.gov/itl/ssd/software-quality-group/computer-forensics-tool-testing-program-cftt)",
    "NIST SP 800-86 — Acquisition hardware requirements",
  ],
};

/** @type {Guide} */
export const acquireAllGuide = {
  title: "Acquire All Guide",
  icon: "◉",
  steps: [
    {
      title: "Prepare evidence storage",
      description:
        "Choose an output folder on a dedicated evidence volume with free space at least equal to the source drive size plus 10% overhead. Use NTFS, APFS, or ext4 — avoid FAT32 for images larger than 4 GB unless you enable split (e.g., 4096 MB). CollectionLoom streams sector-by-sector with no application-level size limit.",
    },
    {
      title: "Detect and select modules",
      description:
        "Click **Detect Devices** to refresh disk, RAM tool, network interface, and mobile lists. Enable only the modules you need (Disk, RAM, Network, Mobile). Each module runs in order; failures in one module do not stop the rest.",
    },
    {
      title: "Enable write protection before disk imaging",
      description:
        "When Disk is enabled, select the source device. CollectionLoom checks hardware blockers automatically (green titlebar badge). If inactive, click **Enable Software Write-Blocker** before starting — Linux BLKROSET, macOS unmount + raw disk path, Windows read-only IOCTL. Acquire All enables software blocking automatically when disk imaging starts if protection is not yet active.",
      warning:
        "Do not start disk acquisition on a mounted read-write volume. Software blocking on Windows requires Administrator privileges.",
    },
    {
      title: "Configure split for large drives",
      description:
        "For drives over 4 GB (or multi-terabyte sources), set **Split (MB)** to 4096 or higher so each segment stays within filesystem limits and is easier to copy. Leave at 0 for a single contiguous image when the destination supports large files. Progress shows human-readable capacity (TB/GB) during imaging.",
    },
    {
      title: "Run batch acquisition and verify",
      description:
        "Click **Start Acquire All**. Disk imaging runs first when enabled, then RAM, network, and mobile in sequence. Monitor per-module progress. After completion, record SHA-256 hashes from disk/RAM outputs in the Chain of Custody tab and export the case bundle when ready.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Integrated digital evidence collection",
    "NIST SP 800-86 — Live response and volatile data ordering",
    "NIST CFReDS — https://cfreds.nist.gov",
  ],
};

/** @type {Guide} */
export const snapshotGuide = {
  title: "Snapshot Guide",
  icon: "◈",   // snapshot
  steps: [
    {
      title: "Take baseline snapshot of clean system",
      description:
        "Before executing any potentially volatile tool, capture a baseline of the running system: list running processes (`ps aux`), active network connections (`ss -tulpn`), loaded kernel modules (`lsmod`), and open file handles (`lsof`). Save outputs to a dated directory under the case evidence folder.",
    },
    {
      title: "Execute target action or tool",
      description:
        "Run the forensic tool or the application under test. Note the exact command, start time (UTC), and any parameters used. If the tool modifies system state (loads a kernel module, writes a log), record that as a side effect.",
    },
    {
      title: "Take post-execution snapshot",
      description:
        "Immediately after the tool completes, re-run the same information-gathering commands from step 1. Capture all outputs fresh — do not reuse the baseline files. Record the end time (UTC).",
    },
    {
      title: "Analyze the delta",
      description:
        "Diff the baseline and post-execution snapshots: compare process lists, network connections, loaded modules, and file handles. Identify new or terminated processes, opened ports, loaded kernel modules, and any file-system writes. Document all changes in the case report.",
    },
  ],
  references: [
    "NIST SP 800-86 — Live forensic data collection and state change analysis",
    "ISO/IEC 27037:2012 — State capture and documentation",
    "SANS — Forensic Analysis of System State Snapshots",
  ],
};

/** @type {Guide} */
export const verificationGuide = {
  title: "Evidence Verification Guide",
  icon: "✓",   // verify
  steps: [
    {
      title: "Select verification algorithm",
      description:
        "Choose a cryptographic hash algorithm for integrity verification. SHA-256 is the recommended minimum (NIST SP 800-131A). Avoid MD5 and SHA-1 for new cases unless required for legacy interoperability. The same algorithm must be used for both source and image hashing.",
    },
    {
      title: "Provide the expected hash value",
      description:
        "If an expected hash was recorded during acquisition (e.g., on the CoC form or in a signed hashset file), enter it in the 'Expected Hash' field. This value is compared against the computed hash of the acquired image to confirm integrity.",
    },
    {
      title: "Compute hash of evidence file",
      description:
        "Select the evidence file or device and run the verification. The tool computes the hash of the selected item and compares it against the expected value. A match confirms integrity; a mismatch indicates tampering, corruption, or a copy error and invalidates the evidence.",
      warning:
        "A hash mismatch does NOT automatically mean intentional tampering — it can also result from incomplete copies, disk errors, or bit-rot on storage media. Investigate the cause before concluding evidence integrity is compromised.",
    },
    {
      title: "Document verification result",
      description:
        "Record the verification result (passed/failed), computed hash, expected hash, algorithm, timestamp, and operator name in the CoC. A passed verification may also be printed and signed for physical chain-of-custody documentation.",
    },
  ],
  references: [
    "NIST SP 800-86 — Hash verification for forensic integrity",
    "NIST SP 800-131A — Transitioning cryptographic algorithms",
    "ISO/IEC 27037:2012 — Integrity verification requirements",
  ],
};

/** @type {Guide} */
export const encryptionGuide = {
  title: "Encryption Assessment Guide",
  icon: "⚷",   // encryption
  steps: [
    {
      title: "Scan target for encrypted volumes / containers",
      description:
        "Use the detection tool to scan the target system for encrypted volumes, containers, and files. The tool checks for common encryption markers: TrueCrypt/VeraCrypt headers, LUKS partitions, BitLocker volumes, FileVault containers, and encrypted ZIP archives. Review the scan results report.",
    },
    {
      title: "Review findings and identify encryption type",
      description:
        "For each detected encryption container, note the encryption type, algorithm (AES-256, Twofish, Serpent, etc.), and whether key material (recovery keys, key files) is locally cached. For BitLocker in AD-bound environments, check if the recovery key is escrowed in Active Directory.",
    },
    {
      title: "Act on recommendations",
      description:
        "Based on the scan, take recommended actions: (a) if a recovery key is available, decrypt the volume and acquire the plaintext; (b) if the volume is unlocked by a logged-in user, perform live acquisition before shutdown; (c) if no key is obtainable, document the encryption as an obstacle and seal the device for forensic imaging with a note of encryption status.",
      warning:
        "Never attempt brute-force or dictionary attacks on encrypted volumes unless explicitly authorised by the investigating authority. Doing so may alter the volume metadata and destroy evidence of decryption attempts.",
    },
  ],
  references: [
    "ISO/IEC 27037:2012 — Encrypted evidence handling",
    "NIST SP 800-86 — Encryption and key recovery considerations",
    "NIST SP 800-88 Rev 1 — Guidelines for Media Sanitization (encryption disposal context)",
  ],
};
