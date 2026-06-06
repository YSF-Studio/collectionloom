CollectionLoom Portable Forensic Kit — tools/
==============================================

Place pre-positioned external binaries here for zero-install field use.
CollectionLoom resolves ./tools/ BEFORE system PATH.

Recommended layout
------------------
  tools/
  ├── manifest.json       SHA-256 hashes (optional but recommended)
  ├── avml                  Linux RAM capture (static binary)
  ├── avml.exe              Windows RAM capture
  ├── winpmem_mini_x64_rc2.exe
  ├── adb / adb.exe
  ├── idevice_id
  ├── idevicebackup2
  └── lime/
      ├── lime-6.2.ko
      └── lime-6.5.ko

manifest.json example
---------------------
{
  "avml": { "file": "avml", "sha256": "abc123..." },
  "adb": { "file": "adb", "sha256": "def456..." }
}

Generate hashes: sha256sum tools/avml tools/adb

Kit root layout (USB / forensic drive)
--------------------------------------
  CollectionLoom/
  ├── CollectionLoom.app   (macOS) or collectionloom / CollectionLoom.exe
  ├── tools/               ← this folder
  └── cases/               ← evidence output

Do NOT install apt/brew/choco on suspect machines — copy this folder and run.
