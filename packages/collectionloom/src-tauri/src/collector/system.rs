use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;
use sysinfo::{Disks, System};

pub struct SystemCollector;

impl CollectorModule for SystemCollector {
    fn name(&self) -> &'static str {
        "system"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let mut sys = System::new_all();
        sys.refresh_all();

        let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
        let kernel = System::kernel_version().unwrap_or_else(|| "unknown".into());
        let os_version = System::os_version().unwrap_or_else(|| "unknown".into());
        let arch = std::env::consts::ARCH.to_string();

        let family = if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "linux"
        };

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let cpu_count = sys.cpus().len();
        let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
            / cpu_count.max(1) as f32;

        let uptime = System::uptime();

        let disks_list = Disks::new_with_refreshed_list();
        let disks: Vec<_> = disks_list
            .iter()
            .map(|d| {
                json!({
                    "name": d.name().to_string_lossy(),
                    "mount": d.mount_point().to_string_lossy(),
                    "total_bytes": d.total_space(),
                    "available_bytes": d.available_space(),
                    "file_system": d.file_system().to_string_lossy(),
                })
            })
            .collect();

        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "hostname": hostname,
            "os": {
                "family": family,
                "version": os_version,
                "kernel": kernel,
                "arch": arch,
            },
            "uptime_secs": uptime,
            "cpu": {
                "cores": cpu_count,
                "usage_percent": cpu_usage,
            },
            "memory": {
                "total_bytes": total_mem,
                "used_bytes": used_mem,
            },
            "disks": disks,
        });

        let path = output_dir.join("system.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "system".into(),
                status: CollectorStatus::Success,
                output_path: Some(path),
                items_count: Some(disks.len()),
                duration_ms: 0,
                error: None,
            },
            Err(e) => CollectorResult {
                module: "system".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}
