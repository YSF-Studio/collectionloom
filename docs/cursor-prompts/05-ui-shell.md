# Cursor Prompt: UI Shell — CollectionLoom V1

## Tujuan
Implementasi Svelte 5 UI untuk CollectionLoom: three-pane layout, dark theme macOS-native, routing, dashboard, case management, snapshot runner UI, dan compare viewer.

## Aturan Desain
- **Dark theme** macOS-native, bukan web-app feel.
- **Three-pane layout:**
  - Sidebar kiri (navigasi + case switcher)
  - Main content tengah (workspace sesuai route)
  - Inspector kanan (metadata, detail, hasil) — muncul saat item dipilih
- **Subtle blur & translucency** di sidebar.
- **Pill badges** untuk status (Running/Completed/Partial/Failed).
- **No double titlebar** — pastikan Tauri config menggunakan `decorations: true` atau `titleBarStyle: Overlay` untuk macOS.
- Spacing bersih, radius 8–12px, divider halus, compact untuk data dense.

## Tech Stack Frontend
- Svelte 5 (routing: svelte-spa-router atau @sveltejs/kit), Tailwind CSS v4
- @tauri-apps/api v2 untuk invoke ke Rust backend

## Halaman/Routes

### 1. **Dashboard** `/`
- 4 KPI cards: Total Cases, Recent Snapshots, Pending Compare, Last Export
- Recent activity list (5 snapshot terakhir)
- Quick action buttons: "New Case", "New Snapshot"

### 2. **Cases** `/cases`
- List case dengan kolom: Title, Status, Operator, Created, Snapshots count
- Search + filter (by status, date range, operator)
- Klik → buka case detail (tampilkan list snapshot di case itu)

### 3. **New Case** `/cases/new`
- Form: title, operator name, purpose, timezone, description, tags
- Validasi: title required, timezone required
- Submit → panggil `create_case()` Tauri command

### 4. **Snapshot** `/snapshot/:caseId`
- Pilih capture profile (radio cards: triage_5m, ir_30m, deep_capture)
- Tombol "Start Snapshot"
- Progress view: list module dengan status spinner + icon (✅ ❌ ⏳)
- Setelah selesai: tampilkan snapshot summary + link ke compare/export

### 5. **Snapshot Detail** `/snapshot/:caseId/:snapshotId`
- Metadata: host, os, profile, started, duration, status
- Tabs per domain: System, Process, Network, Autoruns, Users, Logs
- Setiap tab: tabel/table atau list-items dengan filter
- Action buttons: Compare with other snapshot, Export

### 6. **Compare** `/compare/:caseId`
- Pilih 2 snapshot (dropdown A dan B)
- Tombol "Run Compare"
- Hasil: tabs per domain → Added/Removed/Changed
- Setiap item: key, value, severity badge (info/low/medium/high/critical)
- Summary bar: total added/removed/changed + high priority count

### 7. **Export** `/export/:caseId`
- Pilih format: JSON Pack, Markdown Report, ZIP Bundle
- Checklist: include diff? include all artifacts?
- Tombol "Generate Export"
- Setelah selesai: tampilkan path file, size, hash + button "Open Folder"

### 8. **Settings** `/settings`
- Storage path preferences
- Default capture profile
- About section (versi, app name, copyright)

## Komponen Reusable
- `AppShell.svelte` — three-pane layout + sidebar + slot untuk content
- `Sidebar.svelte` — navigasi + case switcher dropdown
- `Inspector.svelte` — panel detail kanan
- `StatusBadge.svelte` — pill badge (completed=green, partial=yellow, failed=red, running=blue)
- `KPICard.svelte` — card untuk dashboard metrics
- `DataTable.svelte` — tabel dengan sort, search, row click
- `ProgressList.svelte` — list item dengan status icon
- `EmptyState.svelte` — ilustrasi + pesan + CTA untuk empty list
- `ErrorState.svelte` — pesan error + retry button
- `LoadingState.svelte` — spinner/skeleton

## Tauri Commands yang Dipanggil (dari Frontend)
```typescript
// Case
invoke('create_case', { title, operator, purpose, timezone, description })
invoke('list_cases', { status?, search? })
invoke('get_case', { caseId })

// Snapshot
invoke('start_snapshot', { caseId, profile })
invoke('list_snapshots', { caseId })
invoke('get_snapshot', { caseId, snapshotId })
invoke('get_snapshot_progress', { snapshotId }) // polling untuk progress

// Compare
invoke('compare_snapshots', { caseId, snapshotAId, snapshotBId })
invoke('list_diffs', { caseId })

// Export
invoke('export_json', { caseId, snapshotId })
invoke('export_markdown', { caseId, snapshotId, includeDiff })
invoke('export_zip', { caseId })
invoke('list_exports', { caseId })
```

## States yang Harus Ditangani
- Loading: spinner/skeleton saat menunggu invoke.
- Empty: list tidak punya data, tampilkan EmptyState.
- Error: invoke gagal, tampilkan pesan + retry.
- Partial: snapshot selesai dengan beberapa module gagal, tampilkan jelas modul mana yang gagal.

## Output yang Diharapkan
- App shell three-pane layout render di window Tauri.
- Sidebar navigasi berfungsi (klik route → content berganti).
- Dashboard menampilkan data dari invoke (atau state mock untuk development).
- Case create flow: form → submit → redirect ke case detail.
- Snapshot runner: pilih profile → start → progress → result.
- Compare page: pilih snapshot → compare → diff view.
- Export page: pilih format → generate → download link.
- Dark theme konsisten di semua halaman.
- Empty/Loading/Error states tidak ada yang missing.

## Catatan
- V1: gunakan mock data awal untuk development (hardcoded JSON) sampai Rust backend siap.
- V1: polling progress snapshot via interval 1 detik.
- Jangan implementasi drag-drop file di V1 (belum di scope).
- Jangan implementasi export actual download — cukup tampilkan path file.
