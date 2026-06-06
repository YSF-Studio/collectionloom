use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;
use sysinfo::Networks;

pub struct NetworkCollector;

impl CollectorModule for NetworkCollector {
    fn name(&self) -> &'static str {
        "network"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let networks = Networks::new_with_refreshed_list();

        let interfaces: Vec<_> = networks
            .iter()
            .map(|(name, data)| {
                json!({
                    "name": name,
                    "received_bytes": data.received(),
                    "transmitted_bytes": data.transmitted(),
                    "packets_received": data.packets_received(),
                    "packets_transmitted": data.packets_transmitted(),
                })
            })
            .collect();

        let connections = collect_connections();

        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "interfaces": interfaces,
            "connections": connections,
            "listening": collect_listening(),
            "arp": collect_arp(),
        });

        let count = interfaces.len() + connections.len();
        let path = output_dir.join("network.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "network".into(),
                status: if connections.is_empty() && cfg!(not(target_os = "linux")) {
                    CollectorStatus::Partial
                } else {
                    CollectorStatus::Success
                },
                output_path: Some(path),
                items_count: Some(count),
                duration_ms: 0,
                error: None,
            },
            Err(e) => CollectorResult {
                module: "network".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}

fn collect_connections() -> Vec<serde_json::Value> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("ss").args(["-tunap"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            return text
                .lines()
                .skip(1)
                .take(500)
                .map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    json!({
                        "raw": line,
                        "state": parts.first().unwrap_or(&""),
                        "local": parts.get(4).unwrap_or(&""),
                        "remote": parts.get(5).unwrap_or(&""),
                    })
                })
                .collect();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("netstat").args(["-an"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            return text
                .lines()
                .filter(|l| l.contains("tcp") || l.contains("udp"))
                .take(500)
                .map(|line| json!({ "raw": line.trim() }))
                .collect();
        }
    }

    vec![]
}

fn collect_listening() -> Vec<serde_json::Value> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        if let Ok(out) = std::process::Command::new("lsof").args(["-i", "-P", "-n"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            return text
                .lines()
                .skip(1)
                .filter(|l| l.contains("LISTEN"))
                .take(200)
                .map(|line| json!({ "raw": line.trim() }))
                .collect();
        }
    }
    vec![]
}

fn collect_arp() -> Vec<serde_json::Value> {
    if let Ok(out) = std::process::Command::new("arp").arg("-a").output() {
        let text = String::from_utf8_lossy(&out.stdout);
        return text
            .lines()
            .map(|line| json!({ "raw": line.trim() }))
            .collect();
    }
    vec![]
}
