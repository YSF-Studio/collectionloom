use super::{CollectorModule, CollectorResult, CollectorStatus};
use chrono::Utc;
use serde_json::json;
use std::path::Path;

pub struct UserCollector;

impl CollectorModule for UserCollector {
    fn name(&self) -> &'static str {
        "users"
    }

    fn collect(&self, output_dir: &Path) -> CollectorResult {
        let mut users = Vec::new();

        if let Ok(who) = std::process::Command::new("who").output() {
            let text = String::from_utf8_lossy(&who.stdout);
            for line in text.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    users.push(json!({
                        "key": format!("user:{}", parts[0]),
                        "username": parts[0],
                        "terminal": parts[1],
                        "login_time": parts.get(2).unwrap_or(&""),
                        "host": parts.get(4).unwrap_or(&""),
                    }));
                }
            }
        }

        if users.is_empty() {
            if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                users.push(json!({
                    "key": format!("user:{user}"),
                    "username": user,
                    "terminal": "current",
                    "login_time": Utc::now().to_rfc3339(),
                }));
            }
        }

        let count = users.len();
        let data = json!({
            "schema_version": "1.0.0",
            "collected_at": Utc::now().to_rfc3339(),
            "users": users,
        });

        let path = output_dir.join("users.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
            Ok(_) => CollectorResult {
                module: "users".into(),
                status: CollectorStatus::Success,
                output_path: Some(path),
                items_count: Some(count),
                duration_ms: 0,
                error: None,
            },
            Err(e) => CollectorResult {
                module: "users".into(),
                status: CollectorStatus::Error,
                output_path: None,
                items_count: None,
                duration_ms: 0,
                error: Some(e.to_string()),
            },
        }
    }
}
