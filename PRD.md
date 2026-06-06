# CollectionLoom — PRD V1 (Final)

## 1. Ringkasan Produk
CollectionLoom adalah aplikasi **portable, cross-platform forensic collector** untuk menangkap snapshot kondisi sistem, membandingkan snapshot, dan menyiapkan paket handover terstruktur untuk analisis lanjutan.

## 2. Masalah Utama yang Diselesaikan
- Setup tool forensik terlalu berat saat respon insiden.
- Format artefak tidak seragam antar endpoint/OS.
- Sulit melakukan baseline vs kondisi terbaru dengan cepat.
- Chain-of-custody manual rentan miss.
- Handover ke analyst/reporting sering duplikasi kerja.

## 3. Tujuan Produk
1. Capture cepat, konsisten, dan repeatable.
2. Snapshot A/B compare yang jelas (Added/Removed/Changed).
3. Artefak tamper-evident (manifest hash + audit log).
4. Ekspor paket bukti siap handover.

## 4. Non-Tujuan (V1)
- Deep investigation canvas (entity graph, advanced timeline reasoning).
- Kolaborasi real-time multi-user.
- SIEM cloud ingestion built-in.

## 5. Persona
- First responder IR
- Digital forensic analyst
- SOC engineer yang butuh triage portable

## 6. Use Cases Kritis
1. Triage endpoint terduga kompromi (snapshot awal).
2. Re-snapshot setelah containment lalu bandingkan delta.
3. Paket evidence+manifest untuk tim investigasi lanjutan.

## 7. Kebutuhan Fungsional
### 7.1 Case Bootstrap
- Buat case dengan metadata minimal: case_id, title, operator, purpose, timezone.

### 7.2 Capture Profiles
- Preset: `triage_5m`, `ir_30m`, `deep_capture`.
- Custom profile berbasis daftar collector modules.

### 7.3 Snapshot Runner
- Jalankan collector modular lintas domain:
  - system (hostname, os, kernel, uptime, hardware)
  - process (daftar proses, ppid, cpu/mem, cmdline)
  - network (connections, listening, dns cache, arp)
  - autoruns/persistence (startup items, services, scheduled tasks)
  - users/sessions (logged-in users, recent logons)
  - selected logs (event/system log excerpt)
- Progress per modul + status partial success.
- Timeout per collector agar tidak blocking.

### 7.4 Snapshot Store
- List/search snapshot.
- Metadata: snapshot_id, case_id, host, os, timestamp, profile, collector_version, integrity_hash.

### 7.5 Compare Engine
- Pilih Snapshot A dan Snapshot B.
- Output kategori:
  - `added`: items hanya ada di snapshot B.
  - `removed`: items hanya ada di snapshot A.
  - `changed`: items ada di kedua snapshot dengan nilai berbeda.
- Filter berdasarkan domain (system/process/network/dll) dan severity heuristic.
- Diff key: setiap collector module menentukan key comparison-nya sendiri.

### 7.6 Integrity & Custody
- Auto SHA-256 per artefak.
- `hash_manifest.json` per snapshot.
- `collector_audit.log` untuk jejak eksekusi per collector module.
- Signed hash chain (opsional, V1 minimal hash list).

### 7.7 Export
- Ekspor:
  - **Normalized JSON pack** — semua data snapshot dalam satu paket schema versioned.
  - **Markdown summary report** — ringkasan case, snapshot, dan diff.
  - **ZIP bundle** — evidence lengkap + manifest + custody log.

## 8. Kebutuhan Non-Fungsional
- **Portable-first**: jalan dari single folder (unzip → run).
- **Offline-capable**: tidak perlu koneksi internet.
- **Cross-platform parity**: fitur inti identik di Windows/macOS/Linux.
- **Deterministic output schema**: versioned (`schema_version`) biar backward-compatible.
- **Low dependency**: collector core Rust, minim linking eksternal.

## 9. UX Flow Utama (Lapangan)
1. Buka app → Buat/Pilih case.
2. Pilih capture profile.
3. Klik **Start Snapshot**.
4. Lihat progress per collector module.
5. (Opsional) Bandingkan dengan baseline snapshot lain.
6. Review diff + filter domain.
7. Export bundle bukti.

Target: **first snapshot ≤ 3 klik** setelah case dipilih.

## 10. Arsitektur Teknis (Stack Tetap)
- **UI Desktop:** Tauri + Svelte
- **Core engine:** Rust (collector module traits)
- **Storage:** file-based per case folder + SQLite index ringan (V1 opsional)
- **Package:** single binary + assets folder

## 11. Struktur Folder Output (Final)
```
<case_folder>/
├── case.json
├── snapshots/
│   └── <snapshot_id>/
│       ├── snapshot_meta.json
│       ├── artifacts/
│       │   ├── system.json
│       │   ├── process.json
│       │   ├── network.json
│       │   └── ...
│       ├── hash_manifest.json
│       └── collector_audit.log
├── diffs/
│   └── <snapshotA>_vs_<snapshotB>.json
└── exports/
    ├── case_report.md
    ├── evidence_pack.json
    └── collection_bundle.zip
```

## 12. KPI V1
- Waktu snapshot triage (profile `triage_5m`) median < 5 menit.
- Kegagalan total run < 5% (partial success per modul diterima).
- 100% snapshot punya hash manifest.
- Compare result dapat dibaca analyst tanpa parsing manual.

## 13. Acceptance Criteria V1
- [x] App berjalan di macOS/Linux/Windows.
- [x] Bisa membuat case, menjalankan snapshot, dan compare A/B.
- [x] Manifest hash + audit log selalu terbentuk.
- [x] Export JSON + MD report + ZIP bundle berhasil.

## 14. V1.5 / V2 Backlog
- Rule-based noise reduction.
- Signed bundle verification (Ed25519).
- Advanced severity heuristics.
- Team reviewer workflow.
- Collector plugin SDK.
