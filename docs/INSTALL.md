# Installing CollectionLoom

CollectionLoom is available in two forms on every supported platform. **Pre-built binaries are not published** — build from source (see [Building from source](#building-from-source) below), then use the artifacts on your machine.

| Mode | Best for | Evidence storage |
|------|----------|------------------|
| **Installed** | Daily analyst workstation | `~/CollectionLoom/cases/` |
| **Portable kit** | USB / field / suspect-adjacent machines | `./cases/` beside the app |

---

## macOS

### Installed (DMG)

After `npm run build:install`, open the DMG under `packages/collectionloom/src-tauri/target/release/bundle/dmg/`.

1. Open the DMG and drag **CollectionLoom** to **Applications**.
2. Launch from Applications. macOS may prompt to allow the app on first run (unsigned builds: System Settings → Privacy & Security → Open Anyway).

Cases and exports are stored under `~/CollectionLoom/cases/`.

### Portable kit

After `npm run build:portable`, extract the zip from `dist/portable/CollectionLoom-*-portable-macos-*.zip`.

1. Extract anywhere (USB drive recommended).
2. Run `./start-collectionloom.sh` or open `CollectionLoom.app` inside the extracted `CollectionLoom/` folder.
3. Copy external tools into `tools/` before field use (see [tools/README.txt](../tools/README.txt)).

The kit includes a `.portable` marker so cases stay in `./cases/acquisitions/` even before you add tools.

---

## Windows

### Installed (NSIS)

After `npm run build:install`, run the NSIS installer from `packages/collectionloom/src-tauri/target/release/bundle/nsis/`.

1. Run the installer — choose **Install for all users** or **Current user** (both are offered).
2. Launch from the Start menu shortcut.

Cases are stored under `%USERPROFILE%\CollectionLoom\cases\`.

### Portable kit

After `npm run build:portable`, extract `dist/portable/CollectionLoom-*-portable-windows-*.zip`.

1. Extract to a folder on USB or disk (e.g. `D:\CollectionLoom\`).
2. Double-click **Start-CollectionLoom.bat** or run `collectionloom.exe`.
3. Place `avml.exe`, `adb.exe`, etc. in `tools\` as needed.

All DLLs required to run are included beside the executable.

---

## Linux

### Installed (deb or AppImage)

Build artifacts are under `packages/collectionloom/src-tauri/target/release/bundle/`.

**Debian/Ubuntu (.deb)**

```bash
sudo dpkg -i collectionloom_*_amd64.deb
sudo apt-get install -f   # if dependencies are missing
collectionloom
```

**AppImage**

```bash
chmod +x CollectionLoom_*_amd64.AppImage
./CollectionLoom_*_amd64.AppImage
```

Installed packages use `~/CollectionLoom/cases/` for case data.

### Portable kit

After `npm run build:portable`, extract `dist/portable/CollectionLoom-*-portable-linux-*.zip`.

1. Extract and run `./start-collectionloom.sh`.
2. The kit includes an AppImage when built on Linux; otherwise a standalone `collectionloom` binary.
3. Add RAM/mobile/network tools under `tools/`.

For AppImage-based kits, the kit root is the folder containing the AppImage (not the read-only mount).

---

## External tools (all modes)

CollectionLoom cannot redistribute licensed third-party binaries. For RAM capture, mobile triage, and similar modules, copy tools into `./tools/`:

- See [tools/README.txt](../tools/README.txt) for layout.
- Copy [tools/manifest.json.example](../tools/manifest.json.example) to `tools/manifest.json` and fill in SHA-256 hashes for verification.

The in-app **Prerequisites** tab shows whether tools were found in `./tools/` or on PATH, and whether you are running in **Portable kit** or **Installed app** mode.

---

## Building from source

```bash
git clone https://github.com/YSF-Studio/collectionloom.git
cd collectionloom
npm install

# Tauri app + platform installers
npm run tauri:build

# Installers + portable zip
npm run build:portable
```

Portable zips are written to `dist/portable/`.

See the [README](../README.md) for development prerequisites (Rust, Node, platform-specific Tauri deps).

---

## Environment variables

| Variable | Effect |
|----------|--------|
| `COLLECTIONLOOM_KIT_ROOT` | Override kit root (path containing `tools/` and `cases/`) |
| `COLLECTIONLOOM_PORTABLE=1` | Force portable case storage under kit root |

These apply to both installed and portable builds when you need a custom layout.
