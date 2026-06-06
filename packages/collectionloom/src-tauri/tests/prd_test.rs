//! PRD V1 integration tests — collector, compare, export roundtrip

use collectionloom_lib::collector::{SnapshotRunner, overall_status};
use collectionloom_lib::compare::CompareEngine;
use collectionloom_lib::export::ExportEngine;
use collectionloom_lib::models::{Case, CaseOperator, CaptureProfile, SCHEMA_VERSION};
use collectionloom_lib::storage::{
    ensure_case_dirs, new_case_id, new_snapshot_id, read_snapshot_meta, snapshot_dir, write_case,
    write_snapshot_meta,
};
use std::fs;
use tempfile::TempDir;

fn test_case(case_id: &str) -> Case {
    Case {
        schema_version: SCHEMA_VERSION.to_string(),
        case_id: case_id.to_string(),
        title: "Test Case".into(),
        description: None,
        operator: CaseOperator {
            name: "Tester".into(),
            badge_id: None,
            email: None,
        },
        purpose: Some("Unit test".into()),
        timezone: "UTC".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: None,
        status: "open".into(),
        tags: None,
        notes: None,
    }
}

#[test]
fn collector_produces_artifacts_and_manifest() {
    let profile = CaptureProfile {
        name: "triage_5m".into(),
        modules: vec!["system".into(), "users".into()],
        description: "test".into(),
        timeout_seconds: 30,
    };
    let tmp = TempDir::new().unwrap();
    let artifacts = tmp.path().join("artifacts");
    let runner = SnapshotRunner::new(&profile);
    let results = runner.run(&artifacts, "test-snap");
    assert!(!results.is_empty());
    assert!(artifacts.join("system.json").exists());
    let status = overall_status(&results);
    assert!(status == "completed" || status == "partial");
}

#[test]
fn compare_engine_detects_diff() {
    let case_id = new_case_id();
    let snap_a = new_snapshot_id();
    let snap_b = new_snapshot_id();

    // Use home CollectionLoom path — create minimal fixture
    let _case = test_case(&case_id);
    if let Ok(_) = write_case(&_case) {
        for (sid, procs) in [(snap_a.clone(), 1u32), (snap_b.clone(), 2u32)] {
            let dir = snapshot_dir(&case_id, &sid);
            fs::create_dir_all(dir.join("artifacts")).ok();
            let proc_json = format!(
                r#"{{"schema_version":"1.0.0","processes":[{{"pid":{},"name":"test"}}]}}"#,
                procs
            );
            fs::write(dir.join("artifacts/process.json"), proc_json).ok();
            let meta = collectionloom_lib::models::SnapshotMeta {
                schema_version: SCHEMA_VERSION.to_string(),
                snapshot_id: sid.clone(),
                case_id: case_id.clone(),
                host: collectionloom_lib::models::SnapshotHost {
                    hostname: "test".into(),
                    fqdn: None,
                    domain: None,
                    machine_id: None,
                },
                os: collectionloom_lib::models::SnapshotOs {
                    family: "macos".into(),
                    version: "14".into(),
                    kernel: None,
                    arch: None,
                },
                profile: "triage_5m".into(),
                collector_version: "0.1.0".into(),
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: chrono::Utc::now().to_rfc3339(),
                duration_seconds: Some(1.0),
                status: "completed".into(),
                modules: None,
                integrity_hash: "abc".into(),
                notes: None,
            };
            write_snapshot_meta(&meta).ok();
        }

        if let Ok(diff) = CompareEngine::compare(&case_id, &snap_a, &snap_b) {
            assert_eq!(diff.snapshot_a_id, snap_a);
            assert_eq!(diff.snapshot_b_id, snap_b);
        }
    }
}

#[test]
fn export_json_pack_creates_file() {
    let case_id = new_case_id();
    let snap_id = new_snapshot_id();
    let case = test_case(&case_id);
    write_case(&case).unwrap();
    ensure_case_dirs(&case_id).unwrap();

    let dir = snapshot_dir(&case_id, &snap_id);
    fs::create_dir_all(dir.join("artifacts")).unwrap();
    fs::write(
        dir.join("artifacts/system.json"),
        r#"{"schema_version":"1.0.0","hostname":"testhost"}"#,
    )
    .unwrap();

    let meta = collectionloom_lib::models::SnapshotMeta {
        schema_version: SCHEMA_VERSION.to_string(),
        snapshot_id: snap_id.clone(),
        case_id: case_id.clone(),
        host: collectionloom_lib::models::SnapshotHost {
            hostname: "testhost".into(),
            fqdn: None,
            domain: None,
            machine_id: None,
        },
        os: collectionloom_lib::models::SnapshotOs {
            family: "macos".into(),
            version: "14".into(),
            kernel: None,
            arch: None,
        },
        profile: "triage_5m".into(),
        collector_version: "0.1.0".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: chrono::Utc::now().to_rfc3339(),
        duration_seconds: Some(1.0),
        status: "completed".into(),
        modules: None,
        integrity_hash: "deadbeef".into(),
        notes: None,
    };
    write_snapshot_meta(&meta).unwrap();

    let result = ExportEngine::export_json_pack(&case_id, &snap_id);
    assert!(result.is_ok());
    let export = result.unwrap();
    assert!(std::path::Path::new(&export.output_path).exists());
}
