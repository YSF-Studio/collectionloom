# Installing CollectionLoom

CollectionLoom is portable-first on macOS, Windows, and Linux. The recommended experience is a self-contained app folder or zip that you can run directly without a traditional installer.

There are three distribution paths:

- **Source build**: clone the repo and build it yourself.
- **Portable build**: the normal no-install experience for field use.
- **Commercial binary**: a ready-to-run paid portable build distributed outside this repository.

| Mode | Best for | Evidence storage |
|------|----------|------------------|
| **Portable kit** | USB / field / suspect-adjacent machines | `./cases/` beside the app |
| **Source build** | Developers and auditors who want to build locally | `./cases/` or your chosen kit root |
| **Commercial binary** | Users who want a no-setup download | Same as portable, depending on how you package it |

---

## macOS

### Portable macOS build

After `npm run build:portable`, extract the zip from `dist/portable/CollectionLoom-*-portable-macos-*.zip`.

1. Extract anywhere, ideally on a USB drive.
2. Run `./start-collectionloom.sh` or open `CollectionLoom.app` inside the extracted `CollectionLoom/` folder.
3. Copy external tools into `tools/` before field use (see [tools/README.txt](../tools/README.txt)).

The kit includes a `.portable` marker so cases stay in `./cases/acquisitions/` even before you add tools.

If you are distributing a commercial binary, ship the same portable layout as a prebuilt package so the user can launch it immediately after download.

---

## Windows

### Portable Windows build

After `npm run build:portable`, extract `dist/portable/CollectionLoom-*-portable-windows-*.zip`.

1. Extract to a folder on USB or disk, for example `D:\CollectionLoom\`.
2. Double-click `Start-CollectionLoom.bat` or run `collectionloom.exe`.
3. Place `avml.exe`, `adb.exe`, and any other external tools in `tools\` as needed.

All DLLs required to run are included beside the executable.

If you are distributing a commercial binary, ship this same portable layout as the paid download.

---

## Linux

### Portable Linux build

After `npm run build:portable`, extract `dist/portable/CollectionLoom-*-portable-linux-*.zip`.

1. Extract and run `./start-collectionloom.sh`.
2. The kit includes an AppImage when built on Linux; otherwise a standalone `collectionloom` binary.
3. Add RAM, mobile, and network tools under `tools/`.

For AppImage-based kits, the kit root is the folder containing the AppImage, not the read-only mount.

If you are distributing a commercial binary, ship the same portable layout as the paid download.

---

## External tools (all modes)

CollectionLoom cannot redistribute every third-party binary. For RAM capture, mobile triage, and similar modules, copy tools into `./tools/`:

- See [tools/README.txt](../tools/README.txt) for layout.
- Copy [tools/manifest.json.example](../tools/manifest.json.example) to `tools/manifest.json` and fill in SHA-256 hashes for verification.

The in-app **Prerequisites** tab shows whether tools were found in `./tools/` or on PATH, and whether you are running in portable mode or a source-built environment.

---

## Building from source

```bash
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom
npm install

# Portable app + bundled tools where available
npm run tauri:build

# Portable zip package
npm run build:portable
```

Portable zips are written to `dist/portable/`.

### Building the commercial package

The commercial packaging workflow is intentionally separate from the normal source build path. It produces the paid portable binary that you can distribute through your own sales channel.

```bash
npm install
npm run build:commercial
```

The resulting artifacts are written to the normal Tauri bundle directories and `dist/portable/` for the portable package.

See the [README](../README.md) for development prerequisites (Rust, Node, platform-specific Tauri deps).

---

## Environment variables

| Variable | Effect |
|----------|--------|
| `COLLECTIONLOOM_KIT_ROOT` | Override kit root (path containing `tools/` and `cases/`) |
| `COLLECTIONLOOM_PORTABLE=1` | Force portable case storage under kit root |

These apply to both portable and source-built layouts when you need a custom folder structure.
