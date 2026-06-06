//! PRD V1 Tauri commands — case, snapshot, compare, export

use crate::collector::{
    build_hash_manifest, overall_status, write_audit_log, SnapshotProgress, SnapshotRunner,
    SNAPSHOT_PROGRESS,
};
use crate::compare::CompareEngine;
use crate::export::ExportEngine;
use crate::models::{
    Case, CaseOperator, ExportResult, ModuleResult, SnapshotHost, SnapshotMeta, SnapshotOs,
    COLLECTOR_VERSION, SCHEMA_VERSION,
};
use crate::storage::{
    list_cases, list_diffs, list_snapshots, new_case_id, new_snapshot_id, read_case,
    read_snapshot_meta, snapshot_dir, write_case, write_diff, write_hash_manifest,
    write_snapshot_meta, sha256_file,
};
use chrono::Utc;
use std::fs;

#[tauri::command]
pub fn create_case(
    title: String,
    operator: String,
    purpose: Option<String>,
    timezone: String,
    description: Option<String>,
) -> Result<Case, String> {
    let now = Utc::now().to_rfc3339();
    let case = Case {
        schema_version: SCHEMA_VERSION.to_string(),
        case_id: new_case_id(),
        title,
        description,
        operator: CaseOperator {
            name: operator,
            badge_id: None,
            email: None,
        },
        purpose,
        timezone,
        created_at: now.clone(),
        updated_at: Some(now),
        status: "open".into(),
        tags: None,
        notes: None,
    };
    write_case(&case)?;
    Ok(case)
}

#[tauri::command]
pub fn list_cases_cmd(status: Option<String>, search: Option<String>) -> Result<Vec<Case>, String> {
    let mut cases = list_cases()?;
    if let Some(s) = status {
        cases.retain(|c| c.status == s);
    }
    if let Some(q) = search {
        let q = q.to_lowercase();
        cases.retain(|c| {
            c.title.to_lowercase().contains(&q)
                || c.operator.name.to_lowercase().contains(&q)
        });
    }
    Ok(cases)
}

#[tauri::command]
pub fn get_case(case_id: String) -> Result<Case, String> {
    read_case(&case_id)
}

#[tauri::command]
pub fn start_snapshot(case_id: String, profile: String) -> Result<SnapshotMeta, String> {
    let _case = read_case(&case_id)?;
    let capture_profile = crate::models::profile_by_name(&profile)
        .ok_or_else(|| format!("Unknown profile: {profile}"))?;

    let snapshot_id = new_snapshot_id();
    let started = Utc::now();
    let snap_dir = snapshot_dir(&case_id, &snapshot_id);
    let artifacts_dir = snap_dir.join("artifacts");
    fs::create_dir_all(&artifacts_dir).map_err(|e| e.to_string())?;

    let runner = SnapshotRunner::new(&capture_profile);
    let results = runner.run(&artifacts_dir, &snapshot_id);
    write_audit_log(&snap_dir, &snapshot_id, &results);

    let manifest = build_hash_manifest(&snapshot_id, &artifacts_dir, &results);
    write_hash_manifest(&case_id, &snapshot_id, &manifest)?;
    let integrity_hash = sha256_file(&snap_dir.join("hash_manifest.json"))?;

    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".into());
    let os_family = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let completed = Utc::now();
    let duration = completed.signed_duration_since(started).num_milliseconds() as f64 / 1000.0;

    let modules: Vec<ModuleResult> = results
        .iter()
        .map(|r| ModuleResult {
            name: r.module.clone(),
            status: format!("{:?}", r.status).to_lowercase(),
            duration_ms: Some(r.duration_ms),
            error: r.error.clone(),
            items_count: r.items_count,
        })
        .collect();

    let meta = SnapshotMeta {
        schema_version: SCHEMA_VERSION.to_string(),
        snapshot_id: snapshot_id.clone(),
        case_id: case_id.clone(),
        host: SnapshotHost {
            hostname,
            fqdn: None,
            domain: None,
            machine_id: None,
        },
        os: SnapshotOs {
            family: os_family.into(),
            version: sysinfo::System::os_version().unwrap_or_else(|| "unknown".into()),
            kernel: sysinfo::System::kernel_version(),
            arch: Some(std::env::consts::ARCH.to_string()),
        },
        profile,
        collector_version: COLLECTOR_VERSION.to_string(),
        started_at: started.to_rfc3339(),
        completed_at: completed.to_rfc3339(),
        duration_seconds: Some(duration),
        status: overall_status(&results).to_string(),
        modules: Some(modules),
        integrity_hash,
        notes: None,
    };

    write_snapshot_meta(&meta)?;
    Ok(meta)
}

#[tauri::command]
pub fn get_snapshot_progress(snapshot_id: String) -> Result<SnapshotProgress, String> {
    let prog = SNAPSHOT_PROGRESS.lock().map_err(|e| e.to_string())?;
    match &*prog {
        Some(p) if p.snapshot_id == snapshot_id => Ok(p.clone()),
        Some(_) => Err("Different snapshot running".into()),
        None => Ok(SnapshotProgress {
            snapshot_id,
            running: false,
            current_module: None,
            completed_modules: vec![],
            percent: 100.0,
        }),
    }
}

#[tauri::command]
pub fn list_snapshots_cmd(case_id: String) -> Result<Vec<SnapshotMeta>, String> {
    list_snapshots(&case_id)
}

#[tauri::command]
pub fn get_snapshot(case_id: String, snapshot_id: String) -> Result<SnapshotMeta, String> {
    read_snapshot_meta(&case_id, &snapshot_id)
}

#[tauri::command]
pub fn compare_snapshots(
    case_id: String,
    snapshot_a_id: String,
    snapshot_b_id: String,
) -> Result<crate::models::DiffResult, String> {
    let diff = CompareEngine::compare(&case_id, &snapshot_a_id, &snapshot_b_id)?;
    write_diff(&case_id, &diff)?;
    Ok(diff)
}

#[tauri::command]
pub fn list_diffs_cmd(case_id: String) -> Result<Vec<crate::models::DiffResult>, String> {
    list_diffs(&case_id)
}

#[tauri::command]
pub fn export_json(case_id: String, snapshot_id: String) -> Result<ExportResult, String> {
    ExportEngine::export_json_pack(&case_id, &snapshot_id)
}

#[tauri::command]
pub fn export_markdown(
    case_id: String,
    snapshot_id: String,
    include_diff: bool,
) -> Result<ExportResult, String> {
    let diff = if include_diff {
        list_diffs(&case_id)?.into_iter().next()
    } else {
        None
    };
    ExportEngine::export_markdown_report(
        &case_id,
        &snapshot_id,
        diff.as_ref(),
    )
}

#[tauri::command]
pub fn export_zip(case_id: String) -> Result<ExportResult, String> {
    ExportEngine::export_zip_bundle(&case_id)
}

#[tauri::command]
pub fn list_exports(case_id: String) -> Result<Vec<ExportResult>, String> {
    let dir = crate::storage::validated_exports_dir(&case_id)?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut exports = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy();
            let export_type = if name.ends_with(".json") {
                "json_pack"
            } else if name.ends_with(".md") {
                "markdown_report"
            } else if name.ends_with(".zip") {
                "zip_bundle"
            } else {
                "unknown"
            };
            let hash = sha256_file(&path).unwrap_or_default();
            exports.push(ExportResult {
                export_type: export_type.into(),
                output_path: path.to_string_lossy().to_string(),
                size_bytes: fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                sha256: hash,
                exported_at: Utc::now().to_rfc3339(),
                error: None,
            });
        }
    }
    Ok(exports)
}

#[tauri::command]
pub fn generate_qr_label(
    evidence_id: String,
    device: String,
    case_name: String,
    operator: Option<String>,
    acquired_at: Option<String>,
    hash_sha256: Option<String>,
) -> Result<Vec<u8>, String> {
    Ok(ysf_core::generate_qr_label(
        &evidence_id,
        &device,
        &case_name,
        operator.as_deref(),
        acquired_at.as_deref(),
        hash_sha256.as_deref(),
    ))
}

#[tauri::command]
pub fn get_capture_packets(output_file: String, limit: u32) -> Result<Vec<serde_json::Value>, String> {
    use std::io::{BufRead, BufReader};

    if !std::path::Path::new(&output_file).exists() {
        return Ok(vec![]);
    }

    let file = fs::File::open(&output_file).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut packets = Vec::new();

    if let Ok(out) = std::process::Command::new("tcpdump")
        .args(["-r", &output_file, "-n", "-c", &limit.to_string()])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            packets.push(serde_json::json!({
                "no": i + 1,
                "time": parts.get(0).unwrap_or(&""),
                "src": parts.get(2).unwrap_or(&""),
                "dst": parts.get(4).unwrap_or(&""),
                "proto": parts.first().unwrap_or(&""),
                "len": parts.last().unwrap_or(&""),
                "raw": line.trim(),
            }));
        }
    } else {
        for (i, line) in reader.lines().take(limit as usize).enumerate() {
            if let Ok(l) = line {
                packets.push(serde_json::json!({
                    "no": i + 1,
                    "raw": l,
                }));
            }
        }
    }

    Ok(packets)
}

#[tauri::command]
pub fn get_capture_stats(output_file: String) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new(&output_file);
    let size = if path.exists() {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    Ok(serde_json::json!({
        "bytes_captured": size,
        "file": output_file,
    }))
}

#[derive(serde::Serialize)]
pub struct CaseSummaryDto {
    #[serde(flatten)]
    pub summary: crate::storage::CaseSummary,
}

#[tauri::command]
pub fn list_case_summaries_cmd() -> Result<Vec<crate::storage::CaseSummary>, String> {
    crate::storage::list_case_summaries()
}
