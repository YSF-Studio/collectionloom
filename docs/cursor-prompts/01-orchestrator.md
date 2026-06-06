# Cursor Prompt: Orchestrator — CollectionLoom V1

## Tujuan
Generate project scaffolding CollectionLoom menggunakan Tauri + Svelte + Rust collector core. Struktur folder, Cargo workspace, dependencies dasar, dan Tauri window shell yang berfungsi.

## Stack Tetap
- **UI Desktop:** Tauri v2 + Svelte 5
- **Core Engine:** Rust (collector module traits)
- **Storage:** file-based case folder + opsional SQLite (rusqlite)
- **Target platform:** macOS, Linux, Windows

## Struktur Folder yang Harus Dibuat
```
collectionloom/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/        (Tauri command handlers)
│   │   ├── collector/       (collector module traits + implementasi)
│   │   ├── models/          (struct Rust: Case, Snapshot, Artifact, dll)
│   │   ├── storage/         (file-based I/O + SQLite wrapper)
│   │   ├── compare/         (diff engine)
│   │   └── export/          (export module)
│   └── icons/
├── src/
│   ├── app.html
│   ├── routes/
│   │   ├── +layout.svelte
│   │   ├── +page.svelte     (Dashboard)
│   │   ├── cases/
│   │   ├── evidence/
│   │   ├── snapshot/
│   │   ├── compare/
│   │   └── export/
│   ├── lib/
│   │   ├── components/      (UI reusable komponen)
│   │   ├── stores/          (Svelte stores)
│   │   ├── types/           (TypeScript types dari schema JSON)
│   │   └── api/             (Tauri invoke wrappers)
│   └── app.css
├── schemas/
│   ├── case.schema.json
│   ├── snapshot_meta.schema.json
│   ├── artifact_manifest.schema.json
│   ├── diff.schema.json
│   └── collector_audit.schema.json
├── PRD.md
├── Cargo.toml
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tsconfig.json
```

## Persyaratan Scaffolding
1. Init Tauri project dengan `npm create tauri-app@latest` (pilih Svelte + TypeScript).
2. Setup Cargo workspace dengan binary crate `collectionloom`.
3. Tambahkan dependencies Rust:
   - `serde`, `serde_json` (serialization)
   - `chrono` (datetime)
   - `sha2` (hashing)
   - `uuid` (snapshot/case ID)
   - `sysinfo` (system info collector)
   - `whoami` (hostname/user)
4. Tambahkan dependencies frontend:
   - Tailwind CSS v4 untuk styling
   - `@tauri-apps/api` untuk invoke
5. Buat Tauri window dengan title "CollectionLoom", ukuran 1280x800, resizable.

## Output yang Diharapkan
- Folder struktur lengkap siap development.
- `cargo build` berhasil tanpa error.
- `npm run dev` menampilkan window Tauri dengan layout tiga panel kosong.

## Catatan
- Jangan tambahkan fitur yang belum diminta di PRD.
- Fokus ke struktur yang bersih, modular, dan extendable.
- File schemas/ sudah ada, jangan tulis ulang.
