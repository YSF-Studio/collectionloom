use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;

pub struct LogCollector;

impl CollectorModule for LogCollector {
    fn name(&self) -> &'static str {
        "logs"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let lines = collect_log_lines();
        let count = lines.len();

        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "lines": lines,
        });

        let path = output_dir.join("logs.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "logs".into(),
                status: if count > 0 {
                    CollectorStatus::Success
                } else {
                    CollectorStatus::Partial
                },
                output_path: Some(path),
                items_count: Some(count),
                duration_ms: 0,
                error: None,
            },
            Err(e) => CollectorResult {
                module: "logs".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}

fn collect_log_lines() -> Vec<serde_json::Value> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("journalctl")
            .args(["-n", "50", "--no-pager"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            return text
                .lines()
                .map(|l| json!({ "source": "journalctl", "line": l }))
                .collect();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("log")
            .args(["show", "--last", "5m", "--style", "compact"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            return text
                .lines()
                .take(50)
                .map(|l| json!({ "source": "log", "line": l }))
                .collect();
        }
    }

    vec![json!({ "note": "Log collection not available on this platform without elevated access" })]
}
