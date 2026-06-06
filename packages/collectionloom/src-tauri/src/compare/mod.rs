//! Snapshot compare engine — A vs B diff per domain

use crate::models::{DiffChange, DiffItem, DiffResult, DiffSummary, DomainDiff, Severity, SCHEMA_VERSION};
use crate::storage::{read_snapshot_meta, snapshot_dir};
use chrono::Utc;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct CompareEngine;

impl CompareEngine {
    pub fn compare(
        case_id: &str,
        snapshot_a_id: &str,
        snapshot_b_id: &str,
    ) -> Result<DiffResult, String> {
        let dir_a = snapshot_dir(case_id, snapshot_a_id);
        let dir_b = snapshot_dir(case_id, snapshot_b_id);

        if !dir_a.exists() {
            return Err(format!("Snapshot A not found: {snapshot_a_id}"));
        }
        if !dir_b.exists() {
            return Err(format!("Snapshot B not found: {snapshot_b_id}"));
        }

        let _meta_a = read_snapshot_meta(case_id, snapshot_a_id).ok();
        let _meta_b = read_snapshot_meta(case_id, snapshot_b_id).ok();

        let mut domains = HashMap::new();

        for domain in ["process", "network", "autoruns", "users"] {
            if let Some(diff) = compare_domain(&dir_a, &dir_b, domain) {
                if !diff.added.is_empty() || !diff.removed.is_empty() || !diff.changed.is_empty() {
                    domains.insert(domain.to_string(), diff);
                }
            }
        }

        if domains.is_empty() {
            if let Some(diff) = compare_system(&dir_a, &dir_b) {
                domains.insert("system".to_string(), diff);
            }
        }

        if domains.is_empty() {
            return Err("No domains could be compared".into());
        }

        let mut total_added = 0;
        let mut total_removed = 0;
        let mut total_changed = 0;
        let mut high_priority = 0;

        for diff in domains.values() {
            total_added += diff.added.len();
            total_removed += diff.removed.len();
            total_changed += diff.changed.len();
            high_priority += diff
                .added
                .iter()
                .chain(diff.removed.iter())
                .filter(|i| matches_severity(i.severity.as_deref(), &["high", "critical"]))
                .count();
            high_priority += diff
                .changed
                .iter()
                .filter(|c| matches_severity(c.severity.as_deref(), &["high", "critical"]))
                .count();
        }

        Ok(DiffResult {
            schema_version: SCHEMA_VERSION.to_string(),
            snapshot_a_id: snapshot_a_id.to_string(),
            snapshot_b_id: snapshot_b_id.to_string(),
            case_id: Some(case_id.to_string()),
            compared_at: Utc::now().to_rfc3339(),
            domains,
            summary: Some(DiffSummary {
                total_added,
                total_removed,
                total_changed,
                high_priority_changes: Some(high_priority),
                domains_with_changes: None,
            }),
        })
    }
}

fn matches_severity(sev: Option<&str>, targets: &[&str]) -> bool {
    sev.map(|s| targets.contains(&s)).unwrap_or(false)
}

fn load_artifact(dir: &Path, name: &str) -> Option<Value> {
    let path = dir.join("artifacts").join(name);
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn compare_domain(dir_a: &Path, dir_b: &Path, domain: &str) -> Option<DomainDiff> {
    let file = format!("{domain}.json");
    let a = load_artifact(dir_a, &file)?;
    let b = load_artifact(dir_b, &file)?;

    match domain {
        "process" => Some(compare_processes(&a, &b)),
        "network" => Some(compare_network(&a, &b)),
        "autoruns" => Some(compare_autoruns(&a, &b)),
        "users" => Some(compare_users(&a, &b)),
        _ => None,
    }
}

fn compare_system(dir_a: &Path, dir_b: &Path) -> Option<DomainDiff> {
    let a = load_artifact(dir_a, "system.json")?;
    let b = load_artifact(dir_b, "system.json")?;
    let mut changed = Vec::new();

    for field in ["uptime_secs", "kernel"] {
        let old_v = a.get(field).or_else(|| a.pointer(&format!("/os/{field}")));
        let new_v = b.get(field).or_else(|| b.pointer(&format!("/os/{field}")));
        if old_v != new_v {
            changed.push(DiffChange {
                key: field.to_string(),
                old_value: old_v.cloned().unwrap_or(Value::Null),
                new_value: new_v.cloned().unwrap_or(Value::Null),
                severity: Some(Severity::Medium.as_str().to_string()),
            });
        }
    }

    Some(DomainDiff {
        added: vec![],
        removed: vec![],
        changed,
    })
}

fn compare_processes(a: &Value, b: &Value) -> DomainDiff {
    let procs_a = index_by_key(a.get("processes").and_then(|v| v.as_array()), |p| {
        p.get("pid").map(|v| format!("pid:{}", v)).unwrap_or_default()
    });
    let procs_b = index_by_key(b.get("processes").and_then(|v| v.as_array()), |p| {
        p.get("pid").map(|v| format!("pid:{}", v)).unwrap_or_default()
    });
    diff_maps(&procs_a, &procs_b, |key, _| {
        if key.contains("base64") || key.len() > 40 {
            Severity::High.as_str().to_string()
        } else {
            Severity::Info.as_str().to_string()
        }
    })
}

fn compare_network(a: &Value, b: &Value) -> DomainDiff {
    let conns_a = index_by_key(a.get("connections").and_then(|v| v.as_array()), |c| {
        c.get("raw").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    let conns_b = index_by_key(b.get("connections").and_then(|v| v.as_array()), |c| {
        c.get("raw").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    diff_maps(&conns_a, &conns_b, |_, _| Severity::Medium.as_str().to_string())
}

fn compare_autoruns(a: &Value, b: &Value) -> DomainDiff {
    let items_a = index_by_key(a.get("autoruns").and_then(|v| v.as_array()), |i| {
        i.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    let items_b = index_by_key(b.get("autoruns").and_then(|v| v.as_array()), |i| {
        i.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    diff_maps(&items_a, &items_b, |_, _| Severity::High.as_str().to_string())
}

fn compare_users(a: &Value, b: &Value) -> DomainDiff {
    let users_a = index_by_key(a.get("users").and_then(|v| v.as_array()), |u| {
        u.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    let users_b = index_by_key(b.get("users").and_then(|v| v.as_array()), |u| {
        u.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string()
    });
    diff_maps(&users_a, &users_b, |_, _| Severity::Critical.as_str().to_string())
}

fn index_by_key<F>(arr: Option<&Vec<Value>>, key_fn: F) -> HashMap<String, Value>
where
    F: Fn(&Value) -> String,
{
    let mut map = HashMap::new();
    if let Some(items) = arr {
        for item in items {
            let key = key_fn(item);
            if !key.is_empty() {
                map.insert(key, item.clone());
            }
        }
    }
    map
}

fn diff_maps<F>(a: &HashMap<String, Value>, b: &HashMap<String, Value>, sev_fn: F) -> DomainDiff
where
    F: Fn(&str, &Value) -> String,
{
    let keys_a: HashSet<_> = a.keys().cloned().collect();
    let keys_b: HashSet<_> = b.keys().cloned().collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for key in keys_b.difference(&keys_a) {
        if let Some(val) = b.get(key) {
            added.push(DiffItem {
                key: key.clone(),
                value: val.clone(),
                severity: Some(sev_fn(key, val)),
            });
        }
    }

    for key in keys_a.difference(&keys_b) {
        if let Some(val) = a.get(key) {
            removed.push(DiffItem {
                key: key.clone(),
                value: val.clone(),
                severity: Some(Severity::Info.as_str().to_string()),
            });
        }
    }

    for key in keys_a.intersection(&keys_b) {
        if let (Some(old_v), Some(new_v)) = (a.get(key), b.get(key)) {
            if old_v != new_v {
                changed.push(DiffChange {
                    key: key.clone(),
                    old_value: old_v.clone(),
                    new_value: new_v.clone(),
                    severity: Some(sev_fn(key, new_v)),
                });
            }
        }
    }

    DomainDiff {
        added,
        removed,
        changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn diff_detects_added_process() {
        let a = json!({"processes": [{"pid": 1, "name": "init"}]});
        let b = json!({"processes": [{"pid": 1, "name": "init"}, {"pid": 99, "name": "evil"}]});
        let procs_a = index_by_key(a.get("processes").and_then(|v| v.as_array()), |p| {
            format!("pid:{}", p.get("pid").unwrap())
        });
        let procs_b = index_by_key(b.get("processes").and_then(|v| v.as_array()), |p| {
            format!("pid:{}", p.get("pid").unwrap())
        });
        let diff = diff_maps(&procs_a, &procs_b, |_, _| "info".into());
        assert_eq!(diff.added.len(), 1);
    }
}
