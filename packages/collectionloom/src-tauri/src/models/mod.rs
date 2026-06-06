//! CollectionLoom PRD data models

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1.0.0";
pub const COLLECTOR_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseOperator {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub schema_version: String,
    pub case_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub operator: CaseOperator,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    pub timezone: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotHost {
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotOs {
    pub family: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub schema_version: String,
    pub snapshot_id: String,
    pub case_id: String,
    pub host: SnapshotHost,
    pub os: SnapshotOs,
    pub profile: String,
    pub collector_version: String,
    pub started_at: String,
    pub completed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<ModuleResult>>,
    pub integrity_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
    pub collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashManifest {
    pub schema_version: String,
    pub snapshot_id: String,
    pub manifest_created_at: String,
    pub entries: Vec<ManifestEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffItem {
    pub key: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDiff {
    pub added: Vec<DiffItem>,
    pub removed: Vec<DiffItem>,
    pub changed: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_added: usize,
    pub total_removed: usize,
    pub total_changed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_priority_changes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains_with_changes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub schema_version: String,
    pub snapshot_a_id: String,
    pub snapshot_b_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    pub compared_at: String,
    pub domains: std::collections::HashMap<String, DomainDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<DiffSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub export_type: String,
    pub output_path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub exported_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureProfile {
    pub name: String,
    pub modules: Vec<String>,
    pub description: String,
    pub timeout_seconds: u32,
}

pub fn default_profiles() -> Vec<CaptureProfile> {
    vec![
        CaptureProfile {
            name: "triage_5m".into(),
            modules: vec![
                "system".into(),
                "process".into(),
                "network".into(),
                "autoruns".into(),
                "users".into(),
            ],
            description: "Quick triage — core system state in under 5 minutes".into(),
            timeout_seconds: 30,
        },
        CaptureProfile {
            name: "ir_30m".into(),
            modules: vec![
                "system".into(),
                "process".into(),
                "network".into(),
                "autoruns".into(),
                "users".into(),
                "logs".into(),
            ],
            description: "Incident response — includes log excerpts".into(),
            timeout_seconds: 60,
        },
        CaptureProfile {
            name: "deep_capture".into(),
            modules: vec![
                "system".into(),
                "process".into(),
                "network".into(),
                "autoruns".into(),
                "users".into(),
                "logs".into(),
            ],
            description: "Deep capture — extended timeouts for thorough collection".into(),
            timeout_seconds: 120,
        },
    ]
}

pub fn profile_by_name(name: &str) -> Option<CaptureProfile> {
    default_profiles().into_iter().find(|p| p.name == name)
}
