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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableLayout {
    pub platform: String,
    pub kit_root: Option<String>,
    pub tools_dir: Option<String>,
    pub bundled_tools_dir: Option<String>,
    pub cases_dir: String,
    pub default_acquisition_dir: String,
    pub portable_mode: bool,
    /// `portable` (USB kit with tools/) or `installed` (DMG/MSI/DEB — data in home dir).
    pub distribution_mode: String,
    pub path_separator: String,
}

static KIT_ROOT_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);
static BUNDLED_TOOLS_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Override kit root (tests / explicit COLLECTIONLOOM_KIT_ROOT).
pub fn set_kit_root_override(root: Option<PathBuf>) {
    if let Ok(mut guard) = KIT_ROOT_OVERRIDE.lock() {
        *guard = root;
    }
}

/// Set path to Tauri `resources/tools/` (called from app setup).
pub fn set_bundled_tools_dir(dir: Option<PathBuf>) {
    if let Ok(mut guard) = BUNDLED_TOOLS_OVERRIDE.lock() {
        *guard = dir;
    }
}

pub fn bundled_tools_dir() -> Option<PathBuf> {
    if let Ok(guard) = BUNDLED_TOOLS_OVERRIDE.lock() {
        if let Some(ref p) = *guard {
            if p.is_dir() {
                return Some(p.clone());
            }
        }
    }
    std::env::var("COLLECTIONLOOM_BUNDLED_TOOLS")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
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

fn portable_flag_from_env() -> bool {
    std::env::var("COLLECTIONLOOM_PORTABLE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

/// Forensic kit root: folder containing app + `tools/` + `cases/`.
pub fn resolve_kit_root() -> Option<PathBuf> {
    if let Some(o) = kit_root_override() {
        return Some(o);
    }
    if let Some(env) = kit_root_from_env() {
        return Some(env);
    }

    // Linux AppImage: real path is outside the read-only mount.
    #[cfg(target_os = "linux")]
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        if let Some(parent) = PathBuf::from(&appimage).parent() {
            return Some(parent.to_path_buf());
        }
    }

    let dir = exe_dir()?;

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

    #[cfg(target_os = "windows")]
    {
        // Tauri may place the exe in a subfolder; prefer parent when ./tools lives there.
        if let Some(parent) = dir.parent() {
            if parent.join("tools").is_dir() && !dir.join("tools").is_dir() {
                return Some(parent.to_path_buf());
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

/// True when running from a forensic USB kit (tools/, env flag, or marker file).
pub fn is_portable_mode() -> bool {
    if portable_flag_from_env() || kit_root_from_env().is_some() {
        return true;
    }
    if let Some(kit) = resolve_kit_root() {
        if kit.join("tools").is_dir() {
            return true;
        }
        if kit.join(".portable").is_file() {
            return true;
        }
    }
    false
}

/// Use kit-relative `cases/` instead of ~/CollectionLoom/cases when portable.
pub fn use_portable_storage() -> bool {
    is_portable_mode()
}

fn home_cases_dir() -> PathBuf {
    #[cfg(unix)]
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("CollectionLoom").join("cases");
    }
    #[cfg(windows)]
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home).join("CollectionLoom").join("cases");
    }
    std::env::temp_dir().join("CollectionLoom").join("cases")
}

fn platform_label() -> &'static str {
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return "unknown";
}

fn path_separator() -> &'static str {
    #[cfg(windows)]
    return "\\";
    #[cfg(not(windows))]
    return "/";
}

/// Create kit `tools/` and `cases/acquisitions/` when in portable mode.
pub fn ensure_kit_directories() -> Result<(), String> {
    if !is_portable_mode() {
        return Ok(());
    }
    let kit = resolve_kit_root().ok_or("Cannot resolve kit root")?;
    std::fs::create_dir_all(kit.join("tools"))
        .map_err(|e| format!("Cannot create tools/: {e}"))?;
    std::fs::create_dir_all(kit.join("cases").join("acquisitions"))
        .map_err(|e| format!("Cannot create cases/: {e}"))?;
    Ok(())
}

/// Default folder for live acquisition outputs (portable kit or temp).
pub fn default_acquisition_dir() -> PathBuf {
    if is_portable_mode() {
        if let Some(kit) = resolve_kit_root() {
            let dir = kit.join("cases").join("acquisitions");
            let _ = std::fs::create_dir_all(&dir);
            return dir;
        }
    }
    std::env::temp_dir().join("collectionloom_acquisition")
}

pub fn join_acquisition_path(filename: &str) -> PathBuf {
    default_acquisition_dir().join(filename)
}

pub fn distribution_mode() -> &'static str {
    if is_portable_mode() {
        "portable"
    } else {
        "installed"
    }
}

pub fn portable_layout() -> PortableLayout {
    let _ = ensure_kit_directories();
    let kit = resolve_kit_root();
    let tools = tools_dir();
    let bundled = bundled_tools_dir();
    let cases = if use_portable_storage() {
        resolve_kit_root()
            .map(|k| k.join("cases"))
            .unwrap_or_else(default_acquisition_dir)
    } else {
        home_cases_dir()
    };
    let _ = std::fs::create_dir_all(&cases);
    PortableLayout {
        platform: platform_label().into(),
        kit_root: kit.as_ref().map(|p| p.to_string_lossy().into_owned()),
        tools_dir: tools.as_ref().map(|p| p.to_string_lossy().into_owned()),
        bundled_tools_dir: bundled.as_ref().map(|p| p.to_string_lossy().into_owned()),
        cases_dir: cases.to_string_lossy().into_owned(),
        default_acquisition_dir: default_acquisition_dir().to_string_lossy().into_owned(),
        portable_mode: is_portable_mode(),
        distribution_mode: distribution_mode().into(),
        path_separator: path_separator().into(),
    }
}

fn bundled_tool_paths(tools: &Path, name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    #[cfg(windows)]
    {
        if !name.ends_with(".exe") {
            paths.push(tools.join(format!("{name}.exe")));
        }
        paths.push(tools.join(name));
    }
    #[cfg(not(windows))]
    {
        paths.push(tools.join(name));
    }
    paths
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

fn system_tool_paths(name: &str) -> Vec<PathBuf> {
    let file = tool_filename(name);
    let mut dirs: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/opt/homebrew/sbin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/local/sbin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        dirs.extend([
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            dirs.push(PathBuf::from(pf));
        }
        if let Ok(pfx) = std::env::var("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(pfx));
        }
        dirs.push(PathBuf::from("C:\\Windows\\System32"));
    }

    if let Ok(path) = std::env::var("PATH") {
        #[cfg(windows)]
        let sep = ';';
        #[cfg(not(windows))]
        let sep = ':';
        for part in path.split(sep).filter(|p| !p.is_empty()) {
            dirs.push(PathBuf::from(part));
        }
    }

    let mut seen = std::collections::HashSet::new();
    let mut paths = Vec::new();
    for dir in dirs {
        if !seen.insert(dir.clone()) {
            continue;
        }
        paths.push(dir.join(&file));
    }

    #[cfg(not(windows))]
    {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !line.is_empty() {
                    paths.push(PathBuf::from(line));
                }
            }
        }
    }

    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg(name).output() {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        paths.push(PathBuf::from(line));
                    }
                }
            }
        }
    }

    paths
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

fn push_tool_candidates(
    candidates: &mut Vec<(PathBuf, &'static str)>,
    tools: &Path,
    source: &'static str,
    name: &str,
    manifest: Option<&ToolManifest>,
) {
    for bundled in bundled_tool_paths(tools, name) {
        if bundled.is_file() && !candidates.iter().any(|(p, _)| p == &bundled) {
            candidates.push((bundled, source));
        }
    }
    let bundled = tools.join(tool_filename(name));
    if bundled.is_file() && !candidates.iter().any(|(p, _)| p == &bundled) {
        candidates.push((bundled, source));
    }
    if let Some(m) = manifest {
        if let Some(entry) = m.tools.get(name) {
            let alt = tools.join(&entry.file);
            if alt.is_file() && !candidates.iter().any(|(p, _)| p == &alt) {
                candidates.push((alt, source));
            }
        }
    }
}

fn load_tool_manifest() -> Option<ToolManifest> {
    if let Some(tools) = tools_dir() {
        if let Some(m) = load_manifest(&tools) {
            return Some(m);
        }
    }
    bundled_tools_dir().and_then(|t| load_manifest(&t))
}

/// Resolve executable: `./tools/` (kit) → bundled resources → PATH.
pub fn resolve_tool(name: &str) -> Option<ResolvedTool> {
    let manifest = load_tool_manifest();
    let mut candidates: Vec<(PathBuf, &'static str)> = vec![];

    if let Some(tools) = tools_dir().filter(|t| t.is_dir()) {
        push_tool_candidates(&mut candidates, &tools, "portable", name, manifest.as_ref());
    }

    if let Some(bundled) = bundled_tools_dir() {
        push_tool_candidates(&mut candidates, &bundled, "bundled", name, manifest.as_ref());
    }

    for path in system_tool_paths(name) {
        if path.is_file() && !candidates.iter().any(|(p, _)| p == &path) {
            candidates.push((path, "path"));
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
        .ok_or_else(|| format!("{name} not found in kit ./tools/, app resources, or PATH"))
}

/// Run a resolved external tool; fails if manifest hash mismatch when hash is specified.
pub fn command(name: &str) -> Result<Command, String> {
    let resolved = resolve_tool(name).ok_or_else(|| format!("{name} not found in kit ./tools/, app resources, or PATH"))?;
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
    pub bundled_tools_dir: Option<String>,
    pub bundled_tools_available: bool,
    pub manifest_loaded: bool,
    pub portable_mode: bool,
    pub distribution_mode: String,
}

pub fn portable_status() -> PortableStatus {
    let kit = resolve_kit_root();
    let tools = tools_dir();
    let bundled = bundled_tools_dir();
    let tools_exists = tools.as_ref().is_some_and(|t| t.is_dir());
    let bundled_exists = bundled.as_ref().is_some_and(|t| t.is_dir());
    let manifest_loaded = tools
        .as_ref()
        .is_some_and(|t| t.join("manifest.json").is_file())
        || bundled
            .as_ref()
            .is_some_and(|t| t.join("manifest.json").is_file());
    PortableStatus {
        kit_root: kit.as_ref().map(|p| p.to_string_lossy().into_owned()),
        tools_dir: tools.as_ref().map(|p| p.to_string_lossy().into_owned()),
        tools_dir_exists: tools_exists,
        bundled_tools_dir: bundled.as_ref().map(|p| p.to_string_lossy().into_owned()),
        bundled_tools_available: bundled_exists,
        manifest_loaded,
        portable_mode: is_portable_mode(),
        distribution_mode: distribution_mode().into(),
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
    fn resolve_bundled_resource_tool() {
        let tmp = std::env::temp_dir().join("cl_bundled_tools_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("avml"), b"#!/bin/sh\necho avml\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(tmp.join("avml"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        set_bundled_tools_dir(Some(tmp.clone()));
        set_kit_root_override(None);
        let r = resolve_tool("avml").expect("avml");
        assert_eq!(r.source, "bundled");
        set_bundled_tools_dir(None);
        let _ = fs::remove_dir_all(&tmp);
    }

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
        set_kit_root_override(None);
    }

    #[test]
    fn portable_mode_when_tools_present() {
        let tmp = std::env::temp_dir().join("cl_portable_mode_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("tools")).unwrap();
        set_kit_root_override(Some(tmp));
        assert!(is_portable_mode());
        ensure_kit_directories().unwrap();
        assert!(cases_dir().unwrap().join("acquisitions").is_dir());
        set_kit_root_override(None);
    }

    #[test]
    fn join_acquisition_path_uses_kit() {
        let tmp = std::env::temp_dir().join("cl_portable_join_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("tools")).unwrap();
        set_kit_root_override(Some(tmp.clone()));
        let p = join_acquisition_path("disk_image.dd");
        assert!(p.to_string_lossy().contains("acquisitions"));
        assert!(p.to_string_lossy().ends_with("disk_image.dd"));
        set_kit_root_override(None);
    }
}
