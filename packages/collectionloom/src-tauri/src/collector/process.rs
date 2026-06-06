use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub struct ProcessCollector;

impl CollectorModule for ProcessCollector {
    fn name(&self) -> &'static str {
        "process"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let processes: Vec<_> = sys
            .processes()
            .iter()
            .map(|(pid, proc_)| {
                json!({
                    "pid": pid.as_u32(),
                    "ppid": proc_.parent().map(|p| p.as_u32()),
                    "name": proc_.name().to_string_lossy(),
                    "cmdline": proc_.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
                    "exe": proc_.exe().map(|p| p.to_string_lossy().to_string()),
                    "cpu_percent": proc_.cpu_usage(),
                    "memory_bytes": proc_.memory(),
                    "status": format!("{:?}", proc_.status()),
                    "user": proc_.user_id().map(|u| format!("{u:?}")),
                })
            })
            .collect();

        let count = processes.len();
        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "processes": processes,
        });

        let path = output_dir.join("process.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "process".into(),
                status: CollectorStatus::Success,
                output_path: Some(path),
                items_count: Some(count),
                duration_ms: 0,
                error: None,
            },
            Err(e) => CollectorResult {
                module: "process".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}
