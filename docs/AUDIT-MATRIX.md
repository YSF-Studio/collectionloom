# CollectionLoom Audit Matrix

This matrix is a status view of the pain points report. It is intentionally split into four states:

- **Valid**: still a real backlog item
- **Already fixed**: source no longer matches the report
- **Design limitation**: intentional scope boundary, not a bug
- **Needs verification**: likely real, but should be confirmed against the target platform or workflow

| # | Area | Reported issue | Status | Notes |
|---|---|---|---|---|
| 1 | RAM | `capture_mrs()` Intel-only on macOS | Already fixed / design limitation | Raw macOS RAM capture is intentionally not supported in the current product flow |
| 2 | RAM | Legacy WinPmem binary | Already fixed | Tooling now targets `winpmem.exe` / WinPmem v4 aliases |
| 3 | Imaging | `list_disks()` placeholder on non-Linux | Already fixed | Disk enumeration is implemented |
| 4 | Imaging | No sparse handling | Already fixed | Raw DD acquisition now avoids writing all-zero ranges as dense output |
| 5 | RAM | AVML needs sudo / elevation UX | Valid | Privilege guidance is clearer, but automatic elevation is still not built in |
| 6 | Mobile | `adb_backup()` deprecated | Already fixed | Logical triage archive flow now replaces the deprecated ADB backup path |
| 7 | Imaging | E01 is not full parity | Valid | Native E01 support exists, but it is still a subset |
| 8 | Snapshot | `scan_filesystem()` depth / Linux-only scope | Valid | Cross-platform snapshot coverage is still limited |
| 9 | Write Blocker | Windows software write blocker returns admin error | Design limitation | This is largely constrained by Windows privilege and driver model |
| 10 | Encryption | macOS Secure Enclave classification is not detailed enough | Valid | Could be more specific for Apple Silicon vs T2 |
| 11 | Encryption | Partial detection for BitLocker To Go / VeraCrypt | Valid | Coverage is incomplete |
| 12 | Archive | Format detection only uses extension | Already fixed | Magic-byte detection now hardens this path |
| 13 | Archive | RAR password support unimplemented | Valid | Password-aware archive loading is not complete |
| 14 | Network | `pcap::Device::list()` not fully tested on Ventura+ | Needs verification | Likely real, but version/platform specific |
| 15 | Cloud | AWS SigV4 omitted | Already fixed | Core SigV4 code exists now |
| 16 | Cloud | Azure/GCP secrets passed via command line | Already fixed | Secrets are loaded from a native file picker and processed server-side |
| 17 | Disk | HPA/DCO placeholder | Already fixed | Core HPA/DCO implementation exists |
| 18 | Frontend | RAM tab calls missing `list_processes` command | Already fixed | The command exists and is wired in |
| 19 | Custody | Hardcoded operator name | Already fixed | CoC PDF now uses the operator supplied from the UI when available |
| 20 | Snapshot | Compare workaround is not a real diff | Already fixed | Legacy compare command now rejects the fake-diff path and directs users to the stored-snapshot workflow |
| 21 | Carving | Only 17 magic signatures | Valid | Enhancement backlog item |
| 22 | NTFS | Parser coverage is partial | Valid | Enhancement backlog item |
| 23 | Report | PDF layout is basic | Valid | Polish / UX backlog item |
| 24 | Preview | PDF & Office unsupported | Valid | Intentional current limitation |
| 25 | Hash Verify | Blake3 missing from command output | Already fixed | Blake3 is exposed in the hashing path |
| 26 | Frontend | Hardcoded 120s RAM timeout | Already fixed / improved | RAM capture timeout now scales with detected system memory, but workflow-specific tuning may still evolve |
| 27 | Testing | No cross-platform integration tests | Valid | Regression risk remains |

## Practical reading guide

- Treat **Valid** items as the active backlog.
- Treat **Already fixed** items as historical notes only.
- Treat **Design limitation** items as documentation material, not bugs.
- Treat **Needs verification** items as the next audit pass.
