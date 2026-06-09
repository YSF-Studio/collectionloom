use serde::Serialize;
use ts_rs::TS;
use std::process::Command;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/EncryptionReport.ts")]
pub struct EncryptionReport {
    pub platform: String,
    pub has_fde: bool,
    #[ts(type = "unknown | null")]
    pub fde_type: Option<FdeType>,
    pub tpm_present: bool,
    pub tpm_version: Option<String>,
    pub secure_boot: Option<String>,
    pub fde_protectors: Vec<String>,
    pub encrypted_partitions: Vec<PartitionInfo>,
    pub recommendations: Vec<String>,
    pub requires_ram_capture: bool,
    pub requires_recovery_key: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum FdeType {
    BitLocker { status: String },
    FileVault { status: String },
    Luks { version: u8, cipher: String },
    VeraCrypt,
    DeviceEncryption,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(
    export,
    export_to = "../../collectionloom/src/lib/generated/EncryptedPartitionInfo.ts",
    rename = "EncryptedPartitionInfo"
)]
pub struct PartitionInfo {
    pub device: String,
    pub mount_point: Option<String>,
    pub file_system: String,
    pub is_encrypted: bool,
    pub encryption_type: Option<String>,
    pub size_bytes: u64,
}

/// Scan all detectable encryption on the current system
pub fn scan_encryption() -> EncryptionReport {
    let platform = if cfg!(target_os = "windows") { "Windows" }
        else if cfg!(target_os = "macos") { "macOS" }
        else if cfg!(target_os = "linux") { "Linux" }
        else { "Unknown" };

    let mut report = EncryptionReport {
        platform: platform.to_string(),
        has_fde: false,
        fde_type: None,
        tpm_present: false,
        tpm_version: None,
        secure_boot: None,
        fde_protectors: vec![],
        encrypted_partitions: vec![],
        recommendations: vec![],
        requires_ram_capture: false,
        requires_recovery_key: false,
    };

    #[cfg(target_os = "windows")]
    scan_windows(&mut report);

    #[cfg(target_os = "macos")]
    scan_macos(&mut report);

    #[cfg(target_os = "linux")]
    scan_linux(&mut report);

    // Generate recommendations
    if report.has_fde {
        report.requires_ram_capture = true;
        report.requires_recovery_key = true;
        report.recommendations.push(
            "⚠️ Full Disk Encryption detected — capture RAM BEFORE shutdown!".into()
        );
        report.recommendations.push(
            "🔑 Request recovery key / password from device owner or IT admin.".into()
        );
    }
    if report.tpm_present {
        report.recommendations.push(
            "🔐 TPM present — BitLocker/LUKS key may be sealed to TPM. RAM capture critical.".into()
        );
    }

    report
}

#[cfg(target_os = "windows")]
fn scan_windows(report: &mut EncryptionReport) {
    // --- BitLocker via manage-bde ---
    if let Ok(output) = Command::new("manage-bde").arg("-status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_volume: Option<String> = None;
        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("volume ") || lower.contains("volume type:") {
                current_volume = Some(line.trim().to_string());
            }
            if lower.contains("conversion status") && lower.contains("fully encrypted") {
                report.has_fde = true;
                report.fde_type = Some(FdeType::BitLocker { status: "Fully Encrypted".into() });
            } else if lower.contains("conversion status") && lower.contains("encryption in progress") {
                report.has_fde = true;
                report.fde_type = Some(FdeType::BitLocker { status: "Encrypting".into() });
            }
            if lower.contains("protection") && lower.contains("on") {
                report.fde_type = Some(FdeType::BitLocker { status: "Protected".into() });
            }
            if lower.contains("volume type") && lower.contains("removable") && lower.contains("protection status") && lower.contains("on") {
                report.encrypted_partitions.push(PartitionInfo {
                    device: current_volume.clone().unwrap_or_else(|| "Removable volume".into()),
                    mount_point: None,
                    file_system: "BitLocker To Go".into(),
                    is_encrypted: true,
                    encryption_type: Some("BitLocker To Go".into()),
                    size_bytes: 0,
                });
            }
        }
    }
    // TPM check
    if let Ok(output) = Command::new("tpmtool").arg("getdeviceinformation").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("TPM Present: true") || stdout.contains("TpmReady: true") {
            report.tpm_present = true;
            report.tpm_version = Some("2.0".into()); // tpmtool only exists on Win10+ with TPM 2.0
        }
    }
    // Secure Boot
    if let Ok(output) = Command::new("powershell").args(["-Command", "Confirm-SecureBootUEFI"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        report.secure_boot = if stdout.contains("True") { Some("Enabled".into()) }
            else if stdout.contains("False") { Some("Disabled".into()) }
            else { None }
    }
}

#[cfg(target_os = "macos")]
fn scan_macos(report: &mut EncryptionReport) {
    if let Ok(output) = Command::new("fdesetup").arg("status").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("FileVault is On") {
            report.has_fde = true;
            report.fde_type = Some(FdeType::FileVault { status: "On".into() });
            report.encrypted_partitions.push(PartitionInfo {
                device: "/".into(),
                mount_point: Some("/".into()),
                file_system: "APFS".into(),
                is_encrypted: true,
                encryption_type: Some("FileVault".into()),
                size_bytes: 0,
            });
        } else if stdout.contains("FileVault is Off") {
            report.fde_type = Some(FdeType::FileVault { status: "Off".into() });
        }
    }
    // T2 chip/Secure Enclave
    if let Ok(output) = Command::new("system_profiler").args(["SPHardwareDataType"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Apple M") || stdout.contains("Apple Silicon") {
            report.tpm_present = true;
            report.tpm_version = Some("Apple Secure Enclave (Apple Silicon)".into());
        } else if stdout.contains("T2") {
            report.tpm_present = true;
            report.tpm_version = Some("Apple Secure Enclave (T2)".into());
        }
    }
    report.secure_boot = Some("Apple Secure Boot".into());
}

#[cfg(target_os = "linux")]
fn scan_linux(report: &mut EncryptionReport) {
    // --- LUKS detection ---
    // Check for LUKS metadata via dmsetup
    let has_luks = if let Ok(output) = Command::new("dmsetup").args(["ls", "--target", "crypt"]).output() {
        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    } else { false };

    let has_luks_alt = std::path::Path::new("/dev/mapper").exists()
        && std::fs::read_dir("/dev/mapper").map(|d| d.count() > 3).unwrap_or(false);

    if has_luks || has_luks_alt {
        report.has_fde = true;

        // Check LUKS version via cryptsetup
        let version = if let Ok(output) = Command::new("cryptsetup").arg("--version").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("cryptsetup 2") { 2u8 } else { 1u8 }
        } else { 2u8 };

        report.fde_type = Some(FdeType::Luks {
            version,
            cipher: "aes-xts-plain64".into(), // Most common, placeholder
        });
    }

    if let Ok(output) = Command::new("lsblk").args(["-J", "-o", "NAME,PATH,FSTYPE,MOUNTPOINT,TYPE,SIZE,MODEL,RM"]).output() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            collect_encrypted_linux_partitions(&json, report);
        }
    }

    // TPM 2.0 via sysfs
    if std::path::Path::new("/sys/class/tpm/tpm0").exists() {
        report.tpm_present = true;
        let version_path = std::path::Path::new("/sys/class/tpm/tpm0/tpm_version_major");
        if let Ok(ver) = std::fs::read_to_string(version_path) {
            report.tpm_version = Some(format!("{}.0", ver.trim()));
        }
    }

    // Secure Boot
    if let Ok(output) = Command::new("mokutil").arg("--sb-state").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        report.secure_boot = if stdout.contains("SecureBoot enabled") {
            Some("Enabled".into())
        } else {
            Some("Disabled".into())
        };
    }
}

#[cfg(target_os = "linux")]
fn collect_encrypted_linux_partitions(json: &serde_json::Value, report: &mut EncryptionReport) {
    let mut seen = std::collections::HashSet::new();
    fn walk(dev: &serde_json::Value, report: &mut EncryptionReport, seen: &mut std::collections::HashSet<String>) {
        if let Some(arr) = dev.as_array() {
            for item in arr {
                walk(item, report, seen);
            }
            return;
        }
        if let Some(blocks) = dev.get("blockdevices").and_then(|v| v.as_array()) {
            for item in blocks {
                walk(item, report, seen);
            }
            return;
        }
        let name = dev.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let path = dev.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let fstype = dev.get("fstype").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let mount = dev.get("mountpoint").and_then(|v| v.as_str()).map(|s| s.to_string());
        let typ = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let size_bytes = dev.get("size")
            .and_then(|v| v.as_str())
            .and_then(parse_size_bytes)
            .unwrap_or(0);
        let key = path.to_string();
        if !key.is_empty() && !seen.insert(key.clone()) {
            return;
        }

        let mut encryption_type = None;
        if fstype.contains("crypto_luks") || fstype.contains("luks") {
            encryption_type = Some("LUKS".into());
        } else if name.to_lowercase().contains("veracrypt")
            || path.to_lowercase().contains("veracrypt")
            || fstype.contains("veracrypt")
        {
            encryption_type = Some("VeraCrypt".into());
        } else if fstype.contains("bitlocker") {
            encryption_type = Some("BitLocker".into());
        }

        if let Some(enc) = encryption_type {
            report.encrypted_partitions.push(PartitionInfo {
                device: if path.is_empty() { name.to_string() } else { path.to_string() },
                mount_point: mount,
                file_system: if fstype.is_empty() { typ.to_string() } else { fstype.clone() },
                is_encrypted: true,
                encryption_type: Some(enc.clone()),
                size_bytes,
            });
            report.has_fde = true;
            if report.fde_type.is_none() {
                report.fde_type = Some(FdeType::Luks { version: 2, cipher: "unknown".into() });
            }
        }

        if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
            for item in children {
                walk(item, report, seen);
            }
        }
    }
    walk(json, report, &mut seen);
}

#[cfg(target_os = "linux")]
fn parse_size_bytes(s: &str) -> Option<u64> {
    let clean = s.trim().to_uppercase();
    if clean.ends_with("G") {
        clean.trim_end_matches('G').parse::<f64>().ok().map(|v| (v * 1_073_741_824f64) as u64)
    } else if clean.ends_with("M") {
        clean.trim_end_matches('M').parse::<f64>().ok().map(|v| (v * 1_048_576f64) as u64)
    } else if clean.ends_with("K") {
        clean.trim_end_matches('K').parse::<f64>().ok().map(|v| (v * 1024f64) as u64)
    } else {
        clean.parse::<u64>().ok()
    }
}
