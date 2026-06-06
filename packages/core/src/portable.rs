//! Portable forensic kit layout — resolve bundled tools beside the application.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifestEntry {
    pub file: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    #[serde(flatten)]
    pub tools: HashMap<String, ToolManifestEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTool {
    pub name: String,
    pub path: String,
    pub source: String,
    pub hash_verified: Option<bool>,
    pub sha256: Option<String>,
}

static KIT_ROOT_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Override kit root (tests / explicit COLLECTIONLOOM_KIT_ROOT).
pub fn set_kit_root_override(root: Option<PathBuf>) {
    if let Ok(mut guard) = KIT_ROOT_OVERRIDE.lock() {
        *guard = root;
    }
}

fn kit_root_override() -> Option<PathBuf> {
    KIT_ROOT_OVERRIDE.lock().ok().and_then(|g| g.clone())
}

fn kit_root_from_env() -> Option<PathBuf> {
    std::env::var("COLLECTIONLOOM_KIT_ROOT")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
}

/// Forensic kit root: folder containing app + `tools/` + `cases/`.
pub fn resolve_kit_root() -> Option<PathBuf> {
    if let Some(o) = kit_root_override() {
        return Some(o);
    }
    if let Some(env) = kit_root_from_env() {
        return Some(env);
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();

    #[cfg(target_os = "macos")]
    {
        if dir.ends_with("MacOS") {
            if let Some(contents) = dir.parent() {
                if contents.file_name().and_then(|n| n.to_str()) == Some("Contents") {
                    if let Some(app_bundle) = contents.parent() {
                        if app_bundle.extension().and_then(|e| e.to_str()) == Some("app") {
                            if let Some(parent) = app_bundle.parent() {
                                return Some(parent.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }

    Some(dir)
}

pub fn tools_dir() -> Option<PathBuf> {
    resolve_kit_root().map(|r| r.join("tools"))
}

pub fn cases_dir() -> Option<PathBuf> {
    resolve_kit_root().map(|r| r.join("cases"))
}

fn tool_filename(name: &str) -> String {
    #[cfg(windows)]
    {
        if name.ends_with(".exe") {
            return name.to_string();
        }
        format!("{name}.exe")
    }
    #[cfg(not(windows))]
    {
        name.to_string()
    }
}

fn command_on_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let probe = Command::new("where").arg(name).output();
    #[cfg(not(windows))]
    let probe = Command::new("which").arg(name).output();
    let output = probe.ok().filter(|o| o.status.success())?;
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if line.is_empty() {
        None
    } else {
        Some(PathBuf::from(line))
    }
}

fn load_manifest(tools: &Path) -> Option<ToolManifest> {
    let path = tools.join("manifest.json");
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(path).map_err(|e| format!("Open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_against_manifest(
    tool_key: &str,
    path: &Path,
    manifest: &ToolManifest,
) -> (Option<bool>, Option<String>) {
    let Some(entry) = manifest.tools.get(tool_key) else {
        return (None, None);
    };
    let Some(expected) = entry.sha256.as_ref().filter(|h| !h.is_empty()) else {
        let hash = sha256_file(path).ok();
        return (None, hash);
    };
    match sha256_file(path) {
        Ok(actual) => {
            let ok = actual.eq_ignore_ascii_case(expected.trim());
            (Some(ok), Some(actual))
        }
        Err(_) => (Some(false), None),
    }
}

/// Resolve executable: `./tools/` first, then PATH. Optional SHA-256 verify via `tools/manifest.json`.
pub fn resolve_tool(name: &str) -> Option<ResolvedTool> {
    let manifest = tools_dir().as_ref().and_then(|t| load_manifest(t));
    let mut candidates: Vec<(PathBuf, &'static str)> = vec![];

    if let Some(tools) = tools_dir() {
        let bundled = tools.join(tool_filename(name));
        if bundled.is_file() {
            candidates.push((bundled, "portable"));
        }
        if let Some(ref m) = manifest {
            if let Some(entry) = m.tools.get(name) {
                let alt = tools.join(&entry.file);
                if alt.is_file() && !candidates.iter().any(|(p, _)| p == &alt) {
                    candidates.push((alt, "portable"));
                }
            }
        }
    }

    if let Some(p) = command_on_path(name) {
        if !candidates.iter().any(|(c, _)| c == &p) {
            candidates.push((p, "path"));
        }
    }

    let (path, source) = candidates.into_iter().next()?;
    let (hash_verified, sha256) = manifest
        .as_ref()
        .map(|m| verify_against_manifest(name, &path, m))
        .unwrap_or((None, sha256_file(&path).ok()));

    Some(ResolvedTool {
        name: name.into(),
        path: path.to_string_lossy().into_owned(),
        source: source.into(),
        hash_verified,
        sha256,
    })
}

pub fn tool_available(name: &str) -> bool {
    resolve_tool(name).is_some()
}

pub fn tool_path(name: &str) -> Result<PathBuf, String> {
    resolve_tool(name)
        .map(|t| PathBuf::from(t.path))
        .ok_or_else(|| format!("{name} not found in ./tools/ or PATH"))
}

/// Run a resolved external tool; fails if manifest hash mismatch when hash is specified.
pub fn command(name: &str) -> Result<Command, String> {
    let resolved = resolve_tool(name).ok_or_else(|| format!("{name} not found in ./tools/ or PATH"))?;
    if resolved.hash_verified == Some(false) {
        return Err(format!(
            "{name} SHA-256 mismatch — replace binary in ./tools/ or update manifest.json"
        ));
    }
    Ok(Command::new(resolved.path))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableStatus {
    pub kit_root: Option<String>,
    pub tools_dir: Option<String>,
    pub tools_dir_exists: bool,
    pub manifest_loaded: bool,
    pub portable_mode: bool,
}

pub fn portable_status() -> PortableStatus {
    let kit = resolve_kit_root();
    let tools = tools_dir();
    let tools_exists = tools.as_ref().is_some_and(|t| t.is_dir());
    let manifest_loaded = tools
        .as_ref()
        .is_some_and(|t| t.join("manifest.json").is_file());
    PortableStatus {
        kit_root: kit.as_ref().map(|p| p.to_string_lossy().into_owned()),
        tools_dir: tools.as_ref().map(|p| p.to_string_lossy().into_owned()),
        tools_dir_exists: tools_exists,
        manifest_loaded,
        portable_mode: tools_exists,
    }
}

#[cfg(unix)]
fn mount_device_for_path(path: &str) -> Option<String> {
    let output = Command::new("df").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .nth(1)?
        .split_whitespace()
        .next()?
        .to_string();
    Some(line)
}

#[cfg(windows)]
fn mount_device_for_path(path: &str) -> Option<String> {
    let letter = Path::new(path)
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.trim_end_matches('\\').to_uppercase())?;
    Some(letter)
}

#[cfg(not(any(unix, windows)))]
fn mount_device_for_path(_path: &str) -> Option<String> {
    None
}

/// True when output and source resolve to the same volume/mount (risk of overwriting evidence).
pub fn same_volume(output_path: &str, source_device: Option<&str>) -> bool {
    let Some(src) = source_device.filter(|s| !s.is_empty()) else {
        return false;
    };
    let out_mount = mount_device_for_path(output_path);
    let src_mount = mount_device_for_path(src);
    match (out_mount, src_mount) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_bundled_tool_first() {
        let tmp = std::env::temp_dir().join("cl_portable_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("tools")).unwrap();
        fs::write(tmp.join("tools").join("adb"), b"#!/bin/sh\necho adb\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(tmp.join("tools").join("adb"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        set_kit_root_override(Some(tmp.clone()));
        let r = resolve_tool("adb").expect("adb");
        assert_eq!(r.source, "portable");
        assert!(r.path.contains("tools"));
    }
}
