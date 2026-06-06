# Cursor Prompt: Collector Core — CollectionLoom V1

## Tujuan
Implementasi Rust collector engine: trait-based modular collector system, collector modules (system, process, network, autoruns, users, logs), snapshot runner, dan integrity hashing.

## Aturan Kode
- Semua collector module implement trait `Collector` atau `CollectorModule`.
- Setiap module independen — satu module gagal tidak menghentikan module lain.
- Output per module: JSON file deterministic dengan schema versioned.
- Wajib ada timeout per module (default 30 detik).
- Wajib ada hash SHA-256 per file output.

## Trait Collector yang Harus Dibuat
```rust
pub struct CollectorResult {
    pub module: String,
    pub status: CollectorStatus,  // Success | Partial | Skipped | Error
    pub output_path: Option<PathBuf>,
    pub items_count: Option<usize>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub trait CollectorModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn collect(&self, output_dir: &Path) -> CollectorResult;
}
```

## Collector Modules Wajib

### 1. SystemCollector
- hostname, fqdn, domain
- OS family, version, kernel, arch
- Uptime, boot time
- CPU model, cores, usage %
- Total RAM, used RAM
- Disk partitions + usage
- Output: `system.json`

### 2. ProcessCollector
- List seluruh proses: pid, ppid, name, cmdline, exe path
- CPU%, memory%, status (running/sleeping/zombie)
- User yang menjalankan
- Output: `process.json`

### 3. NetworkCollector
- Active TCP/UDP connections: local/remote addr+port, state, pid
- Listening ports: addr, port, pid, process name
- ARP table
- DNS cache (jika ada akses)
- Interface list: name, ip, mac, status
- Output: `network.json`

### 4. AutorunCollector
- Linux: systemd services enabled + user systemd, cron jobs, ~/.config/autostart
- macOS: LaunchAgents + LaunchDaemons, login items
- Windows: registry Run keys, Scheduled Tasks, Services (cross-compile later)
- Output: `autoruns.json`

### 5. UserCollector
- Logged-in users: username, session type, login time, host
- Recent logons (last 10)
- Groups per user (jika akses)
- Output: `users.json`

### 6. LogCollector (opsional untuk triage)
- Last 50 lines syslog/journalctl (Linux) atau log stream snippet (macOS)
- Atau file log spesifik yang ditentukan di profile
- Output: `logs.json`

## Snapshot Runner
```rust
pub struct SnapshotRunner {
    modules: Vec<Box<dyn CollectorModule>>,
}

impl SnapshotRunner {
    pub fn new(profile: &CaptureProfile) -> Self;
    pub fn run(&self, output_dir: &Path) -> Vec<CollectorResult>;
}
```

- Fungsi `run()` memanggil setiap module secara sequential (untuk V1).
- Mengumpulkan semua `CollectorResult` dan menulis `collector_audit.log`.
- Partial success: jika 3 dari 5 module berhasil, status snapshot = "partial".
- Jika semua module gagal, status snapshot = "failed".

## Integrity
- Setelah semua module selesai, generate `hash_manifest.json`:
  ```json
  {
    "schema_version": "1.0.0",
    "snapshot_id": "...",
    "manifest_created_at": "ISO8601",
    "entries": [
      { "filename": "system.json", "size_bytes": 1234, "sha256": "abc...", "module": "system", "collected_at": "ISO8601" }
    ],
    "total_files": 6,
    "total_size_bytes": 99999
  }
  ```

## CaptureProfile
```rust
pub struct CaptureProfile {
    pub name: String,
    pub modules: Vec<String>,
    pub description: String,
    pub timeout_seconds: u32,
}
```

Default profiles:
- `triage_5m`: [system, process, network, autoruns, users], timeout 30s
- `ir_30m`: [system, process, network, autoruns, users, logs], timeout 60s
- `deep_capture`: [system, process, network, autoruns, users, logs], timeout 120s

## Output yang Diharapkan
- Semua trait, struct, dan implementasi collector module compile.
- SnapshotRunner.run() menghasilkan folder dengan file JSON + manifest.
- Partial success handling benar secara logic.
- Unit test untuk setiap module dengan mock system (gunakan conditional compilation).

## Catatan
- Cross-compile Windows collector: gunakan conditional compilation `#[cfg(target_os = "windows")]`
- Untuk V1, prioritas: Linux collector stabil > macOS > Windows.
- Jangan panggil command OS yang butuh sudo/root — skip jika tidak ada akses.
