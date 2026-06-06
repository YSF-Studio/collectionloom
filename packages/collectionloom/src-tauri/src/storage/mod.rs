//! File-based case and snapshot storage

use crate::models::{Case, DiffResult, HashManifest, SnapshotMeta, SCHEMA_VERSION};
use std::fs;
use std::path::{Path, PathBuf};

pub fn cases_root() -> PathBuf {
    if ysf_core::use_portable_storage() {
        if let Some(cases) = ysf_core::cases_dir() {
            let _ = fs::create_dir_all(&cases);
            return cases;
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("CollectionLoom")
        .join("cases")
}

/// Reject path traversal and separator characters in storage identifiers.
pub fn validate_storage_id(id: &str, label: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err(format!("Invalid {label}: empty"));
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(format!("Invalid {label}: path separators and '..' are not allowed"));
    }
    Ok(())
}

pub fn case_dir(case_id: &str) -> PathBuf {
    cases_root().join(case_id)
}

pub fn ensure_case_dirs(case_id: &str) -> Result<PathBuf, String> {
    validate_storage_id(case_id, "case_id")?;
    let root = case_dir(case_id);
    for sub in ["snapshots", "diffs", "exports", "logs"] {
        fs::create_dir_all(root.join(sub)).map_err(|e| e.to_string())?;
    }
    Ok(root)
}

/// One acquisition event (disk imaging, RAM capture, etc.).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcquisitionAuditEntry {
    pub timestamp: String,
    pub acquisition_type: String,
    pub source: String,
    pub destination: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default)]
    pub error_sectors: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

fn global_acquisition_audit_path() -> PathBuf {
    cases_root().join("acquisition_audit.jsonl")
}

fn case_acquisition_audit_path(case_id: &str) -> PathBuf {
    case_dir(case_id).join("logs").join("acquisition_audit.jsonl")
}

/// Append acquisition record to case log (if case_id set) and global fallback log.
pub fn append_acquisition_audit(
    case_id: Option<&str>,
    entry: &AcquisitionAuditEntry,
) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(cases_root()).map_err(|e| e.to_string())?;
    let line = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    let mut written = Vec::new();

    if let Some(id) = case_id.filter(|s| !s.is_empty()) {
        validate_storage_id(id, "case_id")?;
        let _ = ensure_case_dirs(id);
        let path = case_acquisition_audit_path(id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{line}").map_err(|e| e.to_string())?;
        written.push(path);
    }

    let global = global_acquisition_audit_path();
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&global)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())?;
    written.push(global);

    Ok(written)
}

pub fn write_case(case: &Case) -> Result<PathBuf, String> {
    let dir = ensure_case_dirs(&case.case_id)?;
    let path = dir.join("case.json");
    let json = serde_json::to_string_pretty(case).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn read_case(case_id: &str) -> Result<Case, String> {
    validate_storage_id(case_id, "case_id")?;
    let path = case_dir(case_id).join("case.json");
    let data = fs::read_to_string(&path).map_err(|e| format!("Case not found: {e}"))?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn list_cases() -> Result<Vec<Case>, String> {
    fs::create_dir_all(cases_root()).map_err(|e| e.to_string())?;
    let mut cases = Vec::new();
    for entry in fs::read_dir(cases_root()).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            let case_path = entry.path().join("case.json");
            if case_path.exists() {
                if let Ok(data) = fs::read_to_string(&case_path) {
                    if let Ok(case) = serde_json::from_str::<Case>(&data) {
                        cases.push(case);
                    }
                }
            }
        }
    }
    cases.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(cases)
}

pub fn snapshot_dir(case_id: &str, snapshot_id: &str) -> PathBuf {
    case_dir(case_id).join("snapshots").join(snapshot_id)
}

pub fn write_snapshot_meta(meta: &SnapshotMeta) -> Result<PathBuf, String> {
    validate_storage_id(&meta.case_id, "case_id")?;
    validate_storage_id(&meta.snapshot_id, "snapshot_id")?;
    let dir = snapshot_dir(&meta.case_id, &meta.snapshot_id);
    fs::create_dir_all(dir.join("artifacts")).map_err(|e| e.to_string())?;
    let path = dir.join("snapshot_meta.json");
    let json = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn read_snapshot_meta(case_id: &str, snapshot_id: &str) -> Result<SnapshotMeta, String> {
    validate_storage_id(case_id, "case_id")?;
    validate_storage_id(snapshot_id, "snapshot_id")?;
    let path = snapshot_dir(case_id, snapshot_id).join("snapshot_meta.json");
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn list_snapshots(case_id: &str) -> Result<Vec<SnapshotMeta>, String> {
    validate_storage_id(case_id, "case_id")?;
    let dir = case_dir(case_id).join("snapshots");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut snaps = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            let meta_path = entry.path().join("snapshot_meta.json");
            if meta_path.exists() {
                if let Ok(data) = fs::read_to_string(&meta_path) {
                    if let Ok(meta) = serde_json::from_str::<SnapshotMeta>(&data) {
                        snaps.push(meta);
                    }
                }
            }
        }
    }
    snaps.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(snaps)
}

pub fn write_hash_manifest(case_id: &str, snapshot_id: &str, manifest: &HashManifest) -> Result<PathBuf, String> {
    validate_storage_id(case_id, "case_id")?;
    validate_storage_id(snapshot_id, "snapshot_id")?;
    let path = snapshot_dir(case_id, snapshot_id).join("hash_manifest.json");
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn read_hash_manifest(case_id: &str, snapshot_id: &str) -> Result<HashManifest, String> {
    validate_storage_id(case_id, "case_id")?;
    validate_storage_id(snapshot_id, "snapshot_id")?;
    let path = snapshot_dir(case_id, snapshot_id).join("hash_manifest.json");
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn write_diff(case_id: &str, diff: &DiffResult) -> Result<PathBuf, String> {
    validate_storage_id(case_id, "case_id")?;
    validate_storage_id(&diff.snapshot_a_id, "snapshot_a_id")?;
    validate_storage_id(&diff.snapshot_b_id, "snapshot_b_id")?;
    let filename = format!("{}_vs_{}.json", diff.snapshot_a_id, diff.snapshot_b_id);
    let path = case_dir(case_id).join("diffs").join(&filename);
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(diff).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn list_diffs(case_id: &str) -> Result<Vec<DiffResult>, String> {
    validate_storage_id(case_id, "case_id")?;
    let dir = case_dir(case_id).join("diffs");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut diffs = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(data) = fs::read_to_string(entry.path()) {
                if let Ok(diff) = serde_json::from_str::<DiffResult>(&data) {
                    diffs.push(diff);
                }
            }
        }
    }
    Ok(diffs)
}

pub fn exports_dir(case_id: &str) -> PathBuf {
    case_dir(case_id).join("exports")
}

pub fn validated_exports_dir(case_id: &str) -> Result<PathBuf, String> {
    validate_storage_id(case_id, "case_id")?;
    Ok(exports_dir(case_id))
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn sha256_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(data))
}

pub fn new_case_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn new_snapshot_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn validate_schema_version(v: &str) -> bool {
    v == SCHEMA_VERSION
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaseSummary {
    pub case: Case,
    pub snapshot_count: usize,
    pub export_count: usize,
    pub diff_count: usize,
    pub case_dir: String,
}

pub fn list_case_summaries() -> Result<Vec<CaseSummary>, String> {
    let cases = list_cases()?;
    let mut out = Vec::new();
    for case in cases {
        let dir = case_dir(&case.case_id);
        let snapshot_count = list_snapshots(&case.case_id).map(|s| s.len()).unwrap_or(0);
        let diff_count = list_diffs(&case.case_id).map(|d| d.len()).unwrap_or(0);
        let export_count = fs::read_dir(exports_dir(&case.case_id))
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        out.push(CaseSummary {
            case,
            snapshot_count,
            export_count,
            diff_count,
            case_dir: dir.to_string_lossy().into_owned(),
        });
    }
    Ok(out)
}
