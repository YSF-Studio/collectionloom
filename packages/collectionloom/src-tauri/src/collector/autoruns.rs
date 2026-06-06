use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;

pub struct AutorunCollector;

impl CollectorModule for AutorunCollector {
    fn name(&self) -> &'static str {
        "autoruns"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let mut items = Vec::new();

        #[cfg(target_os = "macos")]
        {
            for dir in [
                "/Library/LaunchAgents",
                "/Library/LaunchDaemons",
                "/System/Library/LaunchAgents",
                "/System/Library/LaunchDaemons",
            ] {
                items.extend(scan_dir(dir, "launchd"));
            }
            if let Ok(home) = std::env::var("HOME") {
                items.extend(scan_dir(
                    &format!("{home}/Library/LaunchAgents"),
                    "launchd_user",
                ));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(out) = std::process::Command::new("systemctl")
                .args(["list-unit-files", "--state=enabled", "--no-pager", "--no-legend"])
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().take(300) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(name) = parts.first() {
                        items.push(json!({
                            "key": format!("systemd:{name}"),
                            "name": name,
                            "type": "systemd",
                            "path": line.trim(),
                        }));
                    }
                }
            }
            if let Ok(home) = std::env::var("HOME") {
                items.extend(scan_dir(&format!("{home}/.config/autostart"), "autostart"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            items.push(json!({
                "note": "Windows autorun collection requires cross-compile — skipped in V1",
            }));
        }

        let count = items.len();
        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "autoruns": items,
        });

        let path = output_dir.join("autoruns.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "autoruns".into(),
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
                module: "autoruns".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn scan_dir(dir: &str, kind: &str) -> Vec<serde_json::Value> {
    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten().take(200) {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            items.push(json!({
                "key": format!("{kind}:{name}"),
                "name": name,
                "type": kind,
                "path": path,
            }));
        }
    }
    items
}
