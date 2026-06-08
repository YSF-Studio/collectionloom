CollectionLoom Portable Forensic Kit — tools/
==============================================

This folder is the source of truth for external binaries.

`npm run download-tools` downloads every upstream artifact that exists for the
current platform into this folder, then mirrors the contents into
`packages/collectionloom/src-tauri/resources/tools/` for the app bundle.

CollectionLoom resolves `./tools/` BEFORE system PATH.

Recommended layout
------------------
  tools/
  ├── manifest.json       SHA-256 hashes (optional but recommended)
  ├── avml                  Linux RAM capture (static binary)
  ├── avml.exe              Windows RAM capture
  ├── winpmem.exe
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

Platform notes
--------------
  Windows  — tools/ beside CollectionLoom.exe; resolves adb.exe and adb
  Linux    — tools/ beside binary; AppImage uses APPIMAGE path for kit root
  macOS    — kit root is folder containing CollectionLoom.app (not inside bundle)
  Some tools are source-specific and do not have official downloadable artifacts
  on every platform. Those are marked in the download log and may need manual staging.

Environment
-----------
  COLLECTIONLOOM_KIT_ROOT  — override kit root path
  COLLECTIONLOOM_PORTABLE=1 — force portable cases/ storage
  .portable marker file in kit root — enable portable mode without tools/ yet
