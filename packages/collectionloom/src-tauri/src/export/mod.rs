//! Export module — JSON pack, Markdown report, ZIP bundle

use crate::models::{Case, DiffResult, ExportResult, SnapshotMeta, SCHEMA_VERSION};
use crate::storage::{
    exports_dir, read_case, read_hash_manifest, read_snapshot_meta, sha256_bytes, sha256_file,
    snapshot_dir,
};
use chrono::Utc;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub struct ExportEngine;

impl ExportEngine {
    pub fn export_json_pack(case_id: &str, snapshot_id: &str) -> Result<ExportResult, String> {
        let case = read_case(case_id)?;
        let snapshot = read_snapshot_meta(case_id, snapshot_id)?;
        let artifacts_dir = snapshot_dir(case_id, snapshot_id).join("artifacts");

        let mut artifacts = serde_json::Map::new();
        for name in ["system", "process", "network", "autoruns", "users", "logs"] {
            let path = artifacts_dir.join(format!("{name}.json"));
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(data) => match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(v) => {
                            if name == "process" {
                                if let Some(procs) = v.get("processes").and_then(|p| p.as_array()) {
                                    let top: Vec<_> = procs.iter().take(100).cloned().collect();
                                    artifacts.insert(
                                        name.to_string(),
                                        serde_json::json!({ "processes": top }),
                                    );
                                }
                            } else {
                                artifacts.insert(name.to_string(), v);
                            }
                        }
                        Err(e) => {
                            artifacts.insert(
                                name.to_string(),
                                serde_json::json!({ "_error": e.to_string() }),
                            );
                        }
                    },
                    Err(e) => {
                        artifacts.insert(
                            name.to_string(),
                            serde_json::json!({ "_error": e.to_string() }),
                        );
                    }
                }
            }
        }

        let manifest_hash = read_hash_manifest(case_id, snapshot_id)
            .ok()
            .and_then(|m| serde_json::to_string(&m).ok())
            .map(|s| sha256_bytes(s.as_bytes()))
            .unwrap_or_default();

        let pack = serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "export_type": "json_pack",
            "exported_at": Utc::now().to_rfc3339(),
            "case": case,
            "snapshot": snapshot,
            "artifacts": artifacts,
            "integrity": {
                "manifest_hash": manifest_hash,
            },
        });

        let out_dir = exports_dir(case_id);
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let path = out_dir.join("evidence_pack.json");
        let json_str = serde_json::to_string_pretty(&pack).map_err(|e| e.to_string())?;
        fs::write(&path, &json_str).map_err(|e| e.to_string())?;
        let hash = sha256_file(&path)?;

        Ok(ExportResult {
            export_type: "json_pack".into(),
            output_path: path.to_string_lossy().to_string(),
            size_bytes: fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
            sha256: hash,
            exported_at: Utc::now().to_rfc3339(),
            error: None,
        })
    }

    pub fn export_markdown_report(
        case_id: &str,
        snapshot_id: &str,
        diff: Option<&DiffResult>,
    ) -> Result<ExportResult, String> {
        let case = read_case(case_id)?;
        let snapshot = read_snapshot_meta(case_id, snapshot_id)?;
        let md = build_markdown(&case, &snapshot, diff);

        let out_dir = exports_dir(case_id);
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let path = out_dir.join("case_report.md");
        fs::write(&path, &md).map_err(|e| e.to_string())?;
        let hash = sha256_file(&path)?;

        Ok(ExportResult {
            export_type: "markdown_report".into(),
            output_path: path.to_string_lossy().to_string(),
            size_bytes: fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
            sha256: hash,
            exported_at: Utc::now().to_rfc3339(),
            error: None,
        })
    }

    pub fn export_zip_bundle(case_id: &str) -> Result<ExportResult, String> {
        let case_dir = crate::storage::case_dir(case_id);
        if !case_dir.exists() {
            return Err("Case folder not found".into());
        }

        let out_dir = exports_dir(case_id);
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let zip_path = out_dir.join("collection_bundle.zip");

        let file = File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        add_dir_to_zip(&mut zip, &case_dir, &case_dir, options)?;

        zip.finish().map_err(|e| e.to_string())?;
        let hash = sha256_file(&zip_path)?;

        Ok(ExportResult {
            export_type: "zip_bundle".into(),
            output_path: zip_path.to_string_lossy().to_string(),
            size_bytes: fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0),
            sha256: hash,
            exported_at: Utc::now().to_rfc3339(),
            error: None,
        })
    }
}

fn build_markdown(case: &Case, snapshot: &SnapshotMeta, diff: Option<&DiffResult>) -> String {
    let mut md = String::new();
    md.push_str("# CollectionLoom Case Report\n\n");
    md.push_str("## Case Info\n");
    md.push_str(&format!("- **Case ID:** `{}`\n", case.case_id));
    md.push_str(&format!("- **Title:** {}\n", case.title));
    md.push_str(&format!("- **Operator:** {}\n", case.operator.name));
    md.push_str(&format!("- **Created:** {}\n", case.created_at));
    md.push_str(&format!("- **Status:** {}\n\n", case.status));

    md.push_str("## Snapshot Summary\n");
    md.push_str(&format!("- **Snapshot ID:** `{}`\n", snapshot.snapshot_id));
    md.push_str(&format!("- **Host:** {}\n", snapshot.host.hostname));
    md.push_str(&format!(
        "- **OS:** {} {}\n",
        snapshot.os.family, snapshot.os.version
    ));
    md.push_str(&format!("- **Profile:** {}\n", snapshot.profile));
    md.push_str(&format!("- **Started:** {}\n", snapshot.started_at));
    md.push_str(&format!(
        "- **Duration:** {}s\n",
        snapshot.duration_seconds.unwrap_or(0.0)
    ));
    md.push_str(&format!("- **Status:** {}\n\n", snapshot.status));

    if let Some(modules) = &snapshot.modules {
        md.push_str("## Collector Modules\n");
        md.push_str("| Module | Status | Items | Duration |\n");
        md.push_str("|--------|--------|-------|----------|\n");
        for m in modules {
            md.push_str(&format!(
                "| {} | {} | {} | {}ms |\n",
                m.name,
                m.status,
                m.items_count.unwrap_or(0),
                m.duration_ms.unwrap_or(0)
            ));
        }
        md.push('\n');
    }

    md.push_str("## Artifact Integrity\n");
    md.push_str(&format!(
        "- **Manifest hash:** `{}`\n\n",
        snapshot.integrity_hash
    ));

    if let Some(d) = diff {
        if let Some(summary) = &d.summary {
            md.push_str("## Changes\n");
            md.push_str(&format!("### Added ({})\n", summary.total_added));
            md.push_str(&format!("### Removed ({})\n", summary.total_removed));
            md.push_str(&format!("### Changed ({})\n\n", summary.total_changed));
        }
    }

    md.push_str(&format!(
        "\n---\n*Generated by CollectionLoom v{} at {}*\n",
        crate::models::COLLECTOR_VERSION,
        Utc::now().to_rfc3339()
    ));
    md
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<File>,
    base: &Path,
    dir: &Path,
    options: SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path
            .strip_prefix(base)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            if name.contains("exports") && path.file_name().and_then(|n| n.to_str()) == Some("exports") {
                continue;
            }
            add_dir_to_zip(zip, base, &path, options)?;
        } else {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            let mut f = fs::File::open(&path).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            zip.write_all(&buf).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
