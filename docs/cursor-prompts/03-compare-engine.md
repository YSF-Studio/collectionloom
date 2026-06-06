# Cursor Prompt: Compare Engine — CollectionLoom V1

## Tujuan
Implementasi Rust compare engine untuk membandingkan dua snapshot CollectionLoom (A vs B), menghasilkan diff JSON terstruktur dengan Added/Removed/Changed per domain.

## Aturan Umum
- Diff bersifat **deterministic** — input sama, output sama.
- Setiap domain (system, process, network, dll) punya **key compare logic** sendiri.
- Output sesuai schema `diff.schema.json` yang sudah ada di `schemas/`.
- Severity heuristic sederhana (V1): prioritaskan perubahan process/network/persistence.

## Fungsi Inti
```rust
pub struct CompareEngine;

impl CompareEngine {
    pub fn compare(snapshot_a_dir: &Path, snapshot_b_dir: &Path, output_path: &Path) -> Result<DiffResult>;
}
```

## Key Comparison Logic per Domain

### System
- **Key:** hostname + family (contoh: `hostname|family`)
- Compare: os version, kernel, uptime, ram total.
- Yang dianggap "changed": kernel version, uptime turun drastis, ram usage naik >20%.
- Severity: version change = medium, uptime reset = high.

### Process
- **Key:** pid (contoh: `pid:1234`)
- Compare: name, cmdline, cpu%, mem%, status, user.
- added: proses baru yang tidak ada di snapshot A.
- removed: proses yang hilang dari snapshot A.
- changed: pid sama tapi cmdline/status/user berbeda.
- Severity: proses baru dengan nama mencurigakan (base64/hex) = high.

### Network
- **Key:** local_addr|local_port|protocol (contoh: `0.0.0.0|443|tcp`)
- added: port listening baru.
- removed: port yang tidak ada lagi.
- changed: pid/process name berubah pada port yang sama.
- Severity: port listening baru yang mencurigakan (high port, unknown process) = high.

### Autorun
- **Key:** name (contoh: `systemd:ssh.service`)
- added: persistence baru.
- removed: persistence hilang (bisa indikasi cleanup).
- changed: path/command berubah.
- Severity: persistence baru = high.

### Users
- **Key:** username (contoh: `user:root`)
- added: user baru login.
- removed: user tidak login lagi.
- Severity: user baru tidak dikenal = critical.

## Struct Output
```rust
pub struct DiffResult {
    pub schema_version: String, // "1.0.0"
    pub snapshot_a_id: String,
    pub snapshot_b_id: String,
    pub compared_at: String,    // ISO8601
    pub domains: HashMap<String, DomainDiff>,
    pub summary: DiffSummary,
}

pub struct DomainDiff {
    pub added: Vec<DiffItem>,
    pub removed: Vec<DiffItem>,
    pub changed: Vec<DiffChange>,
}

pub struct DiffItem {
    pub key: String,
    pub value: serde_json::Value,
    pub severity: Severity,
}

pub struct DiffChange {
    pub key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub severity: Severity,
}

pub struct DiffSummary {
    pub total_added: usize,
    pub total_removed: usize,
    pub total_changed: usize,
    pub high_priority_changes: usize,
    pub domains_with_changes: usize,
}

pub enum Severity {
    Info, Low, Medium, High, Critical,
}
```

## Filter & Ordering
- Output domain diurutkan berdasarkan jumlah perubahan (descending).
- Dalam satu domain, diurutkan: critical > high > medium > low > info.
- V1: tidak ada noise reduction otomatis (nanti V1.5).
- V1: tidak ada whitelist rules.

## Error Handling
- Jika snapshot A atau B folder tidak ditemukan: return error jelas.
- Jika artifact JSON corrupt: skip file tersebut, catat di log, lanjutkan file lain.
- Minimum: minimal 1 domain berhasil dibandingkan.

## Output yang Diharapkan
- Fungsi `compare()` menulis diff JSON ke `output_path`.
- Diff file sesuai schema `diff.schema.json` 100%.
- Summary terisi benar.
- Unit test dengan 2 snapshot buatan (A dan B dengan perubahan yang diketahui).

## Catatan
- Jangan menggunakan library diff eksternal — implementasi sendiri.
- Key uniqueness adalah tanggung jawab masing-masing domain comparator.
- Untuk V1, semua collector output dari platform yang sama (Linux-Linux, macOS-macOS).
