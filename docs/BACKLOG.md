# CollectionLoom Backlog

This backlog collects the currently relevant pain points after deduplication and a quick source audit.

## P0 - Critical

| Area | Issue | Why it matters | Status |
|---|---|---|---|
| RAM | Linux privilege UX for AVML is still rough | Capture fails without elevation or a clear prompt flow | In progress |
| RAM | Windows RAM capture needs clear signed-driver onboarding | Win 10/11 users need a predictable WinPmem path | In progress |
| Mobile | `adb_backup()` still relies on the deprecated ADB backup API | Android 12+ behavior is inconsistent and often unusable | Done |

## P1 - Significant

| Area | Issue | Why it matters | Status |
|---|---|---|---|
| Imaging | Sparse file handling is still not optimized for zero-filled regions | Wastes time and output storage on large SSDs | Done |
| Imaging | E01 support is a subset, not full EnCase parity | Metadata/compression workflows remain incomplete | Open |
| Snapshot | Filesystem scanning depth and non-Linux coverage are limited | Full-system volatile snapshots are incomplete outside Linux | Open |
| Write Blocker | Windows software write-blocker UX is limited by platform privilege rules | Users need clearer expectations and fallback guidance | Open |
| Encryption | macOS hardware classification can be more specific | Apple Silicon vs T2 can influence triage guidance | Open |
| Encryption | BitLocker To Go and VeraCrypt container coverage is partial | Encryption scan results are not comprehensive | Open |
| Archive | Format detection is extension-first, not magic-byte-first | Renamed files can evade early detection | Done |
| Archive | RAR password handling is still incomplete | Encrypted RAR archives are not fully usable | Open |

## P2 - Moderate

| Area | Issue | Why it matters | Status |
|---|---|---|---|
| Network | macOS packet-capture behavior needs more real-world testing | BPF permissions and device listing can vary by version | Open |
| Cloud | Secret/token handling should avoid command-line exposure | Reduces accidental disclosure risk | Done |
| Disk | HPA/DCO detection can still be improved for edge cases | Hidden-area detection is not universal | Open |
| Custody | Operator values should always come from case data | Prevents report/case mismatches | Done |
| Snapshot | Snapshot comparison workflow should compare real historical snapshots directly | Improves diff credibility | Done |

## P3 - Minor / Technical Debt

| Area | Issue | Why it matters | Status |
|---|---|---|---|
| Carving | Signature library is still small compared with larger forensic suites | Reduces file-type recovery coverage | In progress |
| NTFS | Parser coverage is partial beyond `$MFT` | Limits NTFS artifact extraction depth | In progress |
| Report | PDF report layout is basic | Output quality can be improved | In progress |
| Preview | PDF and Office document preview is still basic | Limits quick inspection workflows | In progress |
| Hash Verify | Additional hash families can still be exposed in the UI | Keeps UI aligned with core capabilities | Done |
| Frontend | Long-running capture timeouts should be format-aware | Better UX for long imaging jobs | In progress |
| Testing | Cross-platform integration coverage is still thin | Higher regression risk for platform-specific paths | Open |

## Already Resolved / No Longer Applicable

| Area | Old report item | What changed |
|---|---|---|
| RAM | `capture_mrs()` Intel-only macOS RAM capture | Raw macOS RAM capture is intentionally not part of the supported flow |
| RAM | Legacy `winpmem_mini_x64_rc2.exe` | Tooling now points at `winpmem.exe` / WinPmem v4 aliases |
| Imaging | `list_disks()` returned an empty vec on non-Linux | Disk enumeration is implemented and used by the UI |
| Frontend | RAM tab called a missing `list_processes` command | The command exists and is wired up |
| Disk | HPA/DCO detection was a placeholder | Core detection is implemented |
| Hash Verify | Blake3 was missing from command output | Blake3 is already exposed in the hashing path |
| Cloud | AWS SigV4 was omitted | SigV4 implementation exists in core |
| Archive | Format detection only used extensions | Magic-byte fallback now hardens detection |
| Custody | Hardcoded operator name in CoC PDF | Operator now comes from UI input when available |
| Snapshot | Legacy compare command produced a fake diff | Command now rejects the legacy path and points to the stored-snapshot workflow |
| Imaging | Zero-filled raw blocks are now sparse-hole aware | Raw DD acquisition skips writing all-zero ranges |
| Mobile | ADB backup API replaced by logical triage archive flow | Android capture now prefers shared storage archive/pull behavior |
| Cloud | Credentials are loaded from a native file picker and never passed as CLI secrets | Cloud snapshot flow keeps secrets server-side |
