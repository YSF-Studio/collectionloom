//! Modular collector engine

mod autoruns;
mod logs;
mod network;
mod process;
mod system;
mod users;

use crate::models::CaptureProfile;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub use autoruns::AutorunCollector;
pub use logs::LogCollector;
pub use network::NetworkCollector;
pub use process::ProcessCollector;
pub use system::SystemCollector;
pub use users::UserCollector;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CollectorStatus {
    Success,
    Partial,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorResult {
    pub module: String,
    pub status: CollectorStatus,
    pub output_path: Option<PathBuf>,
    pub items_count: Option<usize>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub trait CollectorModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn collect(&self, output_dir: &Path) -> CollectorResult;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub module: String,
    pub action: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SnapshotProgress {
    pub snapshot_id: String,
    pub running: bool,
    pub current_module: Option<String>,
    pub completed_modules: Vec<CollectorResult>,
    pub percent: f64,
}

lazy_static::lazy_static! {
    pub static ref SNAPSHOT_PROGRESS: Arc<Mutex<Option<SnapshotProgress>>> =
        Arc::new(Mutex::new(None));
}

pub fn module_for_name(name: &str) -> Option<Box<dyn CollectorModule>> {
    match name {
        "system" => Some(Box::new(SystemCollector)),
        "process" => Some(Box::new(ProcessCollector)),
        "network" => Some(Box::new(NetworkCollector)),
        "autoruns" => Some(Box::new(AutorunCollector)),
        "users" => Some(Box::new(UserCollector)),
        "logs" => Some(Box::new(LogCollector)),
        _ => None,
    }
}

pub struct SnapshotRunner {
    modules: Vec<Box<dyn CollectorModule>>,
    timeout: Duration,
}

impl SnapshotRunner {
    pub fn new(profile: &CaptureProfile) -> Self {
        let modules: Vec<Box<dyn CollectorModule>> = profile
            .modules
            .iter()
            .filter_map(|n| module_for_name(n))
            .collect();
        Self {
            modules,
            timeout: Duration::from_secs(profile.timeout_seconds as u64),
        }
    }

    pub fn run(&self, artifacts_dir: &Path, snapshot_id: &str) -> Vec<CollectorResult> {
        fs::create_dir_all(artifacts_dir).ok();
        let mut results = Vec::new();
        let total = self.modules.len().max(1);

        {
            let mut prog = SNAPSHOT_PROGRESS.lock().unwrap();
            *prog = Some(SnapshotProgress {
                snapshot_id: snapshot_id.to_string(),
                running: true,
                current_module: None,
                completed_modules: vec![],
                percent: 0.0,
            });
        }

        for (i, module) in self.modules.iter().enumerate() {
            let name = module.name();
            {
                let mut prog = SNAPSHOT_PROGRESS.lock().unwrap();
                if let Some(ref mut p) = *prog {
                    p.current_module = Some(name.to_string());
                    p.percent = (i as f64 / total as f64) * 100.0;
                }
            }

            let start = Instant::now();
            let result = run_with_timeout(module.as_ref(), artifacts_dir, self.timeout);
            let mut result = result;
            result.duration_ms = start.elapsed().as_millis() as u64;
            results.push(result.clone());

            {
                let mut prog = SNAPSHOT_PROGRESS.lock().unwrap();
                if let Some(ref mut p) = *prog {
                    p.completed_modules.push(result);
                    p.percent = ((i + 1) as f64 / total as f64) * 100.0;
                }
            }
        }

        {
            let mut prog = SNAPSHOT_PROGRESS.lock().unwrap();
            if let Some(ref mut p) = *prog {
                p.running = false;
                p.current_module = None;
                p.percent = 100.0;
            }
        }

        results
    }
}

fn run_with_timeout(
    module: &dyn CollectorModule,
    output_dir: &Path,
    timeout: Duration,
) -> CollectorResult {
    let name = module.name().to_string();
    let output_dir = output_dir.to_path_buf();
    let name_for_err = name.clone();
    let handle = std::thread::spawn(move || {
        let m = module_for_name(&name).unwrap();
        m.collect(&output_dir)
    });

    match handle.join() {
        Ok(r) => r,
        Err(_) => CollectorResult {
            module: name_for_err,
            status: CollectorStatus::Error,
            output_path: None,
            items_count: None,
            duration_ms: 0,
            error: Some("Module panicked".into()),
        },
    }
}

pub fn write_audit_log(snapshot_dir: &Path, snapshot_id: &str, results: &[CollectorResult]) {
    let entries: Vec<AuditEntry> = results
        .iter()
        .map(|r| AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            module: r.module.clone(),
            action: "finish_collection".into(),
            status: match r.status {
                CollectorStatus::Success => "success".into(),
                CollectorStatus::Partial => "partial".into(),
                CollectorStatus::Skipped => "skipped".into(),
                CollectorStatus::Error => "error".into(),
            },
            message: None,
            duration_ms: Some(r.duration_ms),
            items_count: r.items_count,
            error_detail: r.error.clone(),
        })
        .collect();

    let audit = serde_json::json!({
        "schema_version": "1.0.0",
        "snapshot_id": snapshot_id,
        "entries": entries,
    });

    let path = snapshot_dir.join("collector_audit.log");
    if let Ok(json) = serde_json::to_string_pretty(&audit) {
        let _ = fs::write(path, json);
    }
}

pub fn build_hash_manifest(
    snapshot_id: &str,
    artifacts_dir: &Path,
    results: &[CollectorResult],
) -> crate::models::HashManifest {
    use crate::models::{HashManifest, ManifestEntry, SCHEMA_VERSION};
    use crate::storage::sha256_file;

    let mut entries = Vec::new();
    let mut total_size = 0u64;

    for result in results {
        if let Some(ref path) = result.output_path {
            if path.exists() {
                if let Ok(hash) = sha256_file(path) {
                    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    total_size += size;
                    entries.push(ManifestEntry {
                        filename: format!("artifacts/{}", path.file_name().unwrap().to_string_lossy()),
                        module: Some(result.module.clone()),
                        size_bytes: size,
                        sha256: hash,
                        collected_at: Utc::now().to_rfc3339(),
                    });
                }
            }
        }
    }

    HashManifest {
        schema_version: SCHEMA_VERSION.to_string(),
        snapshot_id: snapshot_id.to_string(),
        manifest_created_at: Utc::now().to_rfc3339(),
        entries: entries.clone(),
        total_files: Some(entries.len()),
        total_size_bytes: Some(total_size),
    }
}

pub fn overall_status(results: &[CollectorResult]) -> &'static str {
    if results.is_empty() {
        return "failed";
    }
    let ok = results
        .iter()
        .filter(|r| r.status == CollectorStatus::Success || r.status == CollectorStatus::Partial)
        .count();
    if ok == 0 {
        "failed"
    } else if ok == results.len() {
        "completed"
    } else {
        "partial"
    }
}
