CollectionLoom — Portable Forensic Kit
======================================

This is a zero-install portable kit for field acquisition. Copy the entire
folder to a USB drive or forensic workstation and run the launcher — no
installer, apt, brew, or choco required on the suspect machine.

Quick start
-----------
  Windows  — double-click Start-CollectionLoom.bat
  macOS    — double-click start-collectionloom.sh (or run: open CollectionLoom.app)
  Linux    — ./start-collectionloom.sh

Kit layout
----------
  CollectionLoom/
  ├── .portable              marker file (enables portable storage mode)
  ├── README-PORTABLE.txt    this file
  ├── CollectionLoom.app     macOS application bundle
  ├── collectionloom.exe     Windows executable (+ DLLs)
  ├── *.AppImage             Linux AppImage (when included)
  ├── start-collectionloom.sh / Start-CollectionLoom.bat
  ├── tools/                 external binaries (avml, adb, etc.)
  │   ├── README.txt
  │   └── manifest.json.example
  └── cases/
      └── acquisitions/      default evidence output folder

External tools
--------------
Place licensed third-party binaries in tools/ before field use. See
tools/README.txt for layout and tools/manifest.json.example for SHA-256
verification. CollectionLoom resolves ./tools/ before system PATH.

Data storage
------------
  Portable kit — cases/ and acquisitions/ live beside the app (USB-safe)
  Installed app — cases/ under ~/CollectionLoom/cases/ (see INSTALL.md)

Environment overrides
---------------------
  COLLECTIONLOOM_KIT_ROOT   override kit root path
  COLLECTIONLOOM_PORTABLE=1 force portable storage mode

Documentation
-------------
  https://github.com/YSF-Studio/collectionloom/blob/main/docs/INSTALL.md
