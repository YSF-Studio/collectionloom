Bundled forensic tools (embedded in the app)
============================================

Binaries are populated at build time by: npm run download-tools

They ship inside the Tauri app bundle (Resources/tools/) — no separate tools/
folder is required for installed builds.

Priority at runtime:
  1. ./tools/ on forensic USB kit (portable override)
  2. Bundled resources (this folder inside the app)
  3. System PATH / Homebrew

Skip download (offline dev): SKIP_TOOL_DOWNLOAD=1 npm run tauri:build
