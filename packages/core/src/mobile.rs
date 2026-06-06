use serde::Serialize;
use ts_rs::TS;

fn validate_mobile_device_id(id: &str) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Empty device ID".into());
    }
    if id.len() > 128 {
        return Err("Device ID too long".into());
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.')) {
        return Err("Invalid device ID characters".into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/MobileDevice.ts")]
pub struct MobileDevice {
    pub id: String,
    pub device_type: String,  // "android" | "ios"
    pub model: String,
    pub serial: String,
}

/// List connected Android devices via ADB
pub fn list_android_devices() -> Result<Vec<MobileDevice>, String> {
    let output = crate::portable::command("adb")?
        .args(["devices", "-l"])
        .output().map_err(|e| format!("ADB not found or failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = vec![];
    for line in stdout.lines().skip(1) { // Skip "List of devices attached"
        if line.trim().is_empty() || line.contains("offline") { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 1 {
            let id = parts[0].to_string();
            let model = parts.iter()
                .find(|s| s.starts_with("model:"))
                .map(|s| s.trim_start_matches("model:").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            devices.push(MobileDevice {
                id,
                device_type: "android".into(),
                model,
                serial: parts[0].to_string(),
            });
        }
    }
    Ok(devices)
}

/// Run ADB backup
pub fn adb_backup(device_id: &str, output_path: &str) -> Result<String, String> {
    validate_mobile_device_id(device_id)?;
    let status = crate::portable::command("adb")?
        .args(["-s", device_id, "backup", "-apk", "-shared", "-all", "-f", output_path])
        .status()
        .map_err(|e| format!("ADB backup failed: {}", e))?;

    if !status.success() { return Err("ADB backup returned non-zero exit".into()); }
    Ok(format!("Backup saved to {}", output_path))
}

/// List paired iOS devices (via idevice_id or iTunes)
pub fn list_ios_devices() -> Result<Vec<MobileDevice>, String> {
    let output = crate::portable::command("idevice_id")?
        .arg("-l")
        .output().map_err(|_| "idevice_id not found — libimobiledevice not installed".to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = vec![];
    for line in stdout.lines() {
        if line.trim().is_empty() { continue; }
        devices.push(MobileDevice {
            id: line.trim().to_string(),
            device_type: "ios".into(),
            model: "iOS Device".into(),
            serial: line.trim().to_string(),
        });
    }
    Ok(devices)
}

/// Run iTunes-style iOS backup (via idevicebackup2)
pub fn ios_backup(device_id: &str, output_path: &str) -> Result<String, String> {
    validate_mobile_device_id(device_id)?;
    let status = crate::portable::command("idevicebackup2")?
        .args(["-u", device_id, "backup", output_path])
        .status()
        .map_err(|e| format!("iOS backup failed: {}", e))?;

    if !status.success() { return Err("iOS backup returned non-zero exit".into()); }
    Ok(format!("Backup saved to {}", output_path))
}
