//! Pre-flight dependency and privilege checks before acquisition.

use chrono::Utc;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/PreflightCategory.ts")]
pub enum PreflightCategory {
    PureRust,
    SystemLibrary,
    ExternalBinary,
    Privilege,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/PreflightCheck.ts")]
pub struct PreflightCheck {
    pub id: String,
    pub name: String,
    pub category: PreflightCategory,
    pub required_for: String,
    pub available: bool,
    pub detail: String,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/PreflightReport.ts")]
pub struct PreflightReport {
    pub platform: String,
    pub checked_at: String,
    pub checks: Vec<PreflightCheck>,
    pub missing_count: u32,
    pub warning_count: u32,
    pub summary: String,
    pub portable: crate::portable::PortableStatus,
}


fn is_elevated() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("net")
            .args(["session"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
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

fn push_pure(checks: &mut Vec<PreflightCheck>, id: &str, name: &str, required_for: &str) {
    checks.push(PreflightCheck {
        id: id.into(),
        name: name.into(),
        category: PreflightCategory::PureRust,
        required_for: required_for.into(),
        available: true,
        detail: "Built into CollectionLoom (pure Rust)".into(),
        install_hint: None,
    });
}

fn check_libpcap() -> PreflightCheck {
    #[cfg(unix)]
    {
        match pcap::Device::list() {
            Ok(devs) => PreflightCheck {
                id: "libpcap".into(),
                name: "libpcap (network capture)".into(),
                category: PreflightCategory::SystemLibrary,
                required_for: "Network Capture".into(),
                available: true,
                detail: format!("{} interface(s) visible via libpcap", devs.len()),
                install_hint: None,
            },
            Err(e) => PreflightCheck {
                id: "libpcap".into(),
                name: "libpcap (network capture)".into(),
                category: PreflightCategory::SystemLibrary,
                required_for: "Network Capture".into(),
                available: false,
                detail: format!("libpcap unavailable: {e}"),
                install_hint: Some(
                    "Linux: libpcap-dev · macOS: Xcode CLT · Windows: install Npcap (npcap.com)"
                        .into(),
                ),
            },
        }
    }
    #[cfg(not(unix))]
    {
        PreflightCheck {
            id: "libpcap".into(),
            name: "Npcap / libpcap (network capture)".into(),
            category: PreflightCategory::SystemLibrary,
            required_for: "Network Capture".into(),
            available: std::path::Path::new("C:\\Windows\\System32\\Npcap\\wpcap.dll").exists()
                || std::path::Path::new("C:\\Windows\\System32\\wpcap.dll").exists(),
            detail: "Windows requires Npcap runtime for live capture".into(),
            install_hint: Some("Install Npcap from https://npcap.com (WinPcap-compatible mode)".into()),
        }
    }
}

fn check_external_tool(id: &str, tool: &str, display: &str, required_for: &str) -> PreflightCheck {
    match crate::portable::resolve_tool(tool) {
        Some(r) => {
            let hash_ok = r.hash_verified != Some(false);
            let mut detail = format!("Found via {} at {}", r.source, r.path);
            if r.hash_verified == Some(true) {
                detail.push_str(" · SHA-256 verified");
            } else if r.hash_verified == Some(false) {
                detail.push_str(" · SHA-256 MISMATCH");
            }
            PreflightCheck {
                id: id.into(),
                name: display.into(),
                category: PreflightCategory::ExternalBinary,
                required_for: required_for.into(),
                available: hash_ok,
                detail,
                install_hint: if r.hash_verified == Some(false) {
                    Some("Replace binary in ./tools/ or update tools/manifest.json".into())
                } else if r.source == "bundled" {
                    None
                } else if r.source == "portable" {
                    Some("Portable kit ./tools/ overrides the app-bundled copy".into())
                } else {
                    Some("Using system PATH; rebuild with npm run download-tools to embed a copy".into())
                },
            }
        }
        None => PreflightCheck {
            id: id.into(),
            name: display.into(),
            category: PreflightCategory::ExternalBinary,
            required_for: required_for.into(),
            available: false,
            detail: format!("{tool} not found in kit ./tools/, app resources, or PATH"),
            install_hint: Some(format!(
                "Rebuild with npm run download-tools, or place {tool} in ./tools/ for portable kits"
            )),
        },
    }
}

fn check_ram_tools() -> Vec<PreflightCheck> {
    let mut checks = vec![];

    #[cfg(target_os = "linux")]
    {
        checks.push(check_external_tool(
            "avml",
            "avml",
            "avml (RAM capture)",
            "RAM Capture (Linux)",
        ));
        let lime_dir = crate::portable::tools_dir().map(|t| t.join("lime"));
        let lime_in_kit = lime_dir.as_ref().is_some_and(|d| d.is_dir());
        let lime = lime_in_kit || std::path::Path::new("/usr/lib/lime/lime.ko").exists();
        checks.push(PreflightCheck {
            id: "lime".into(),
            name: "LiME kernel module (RAM capture)".into(),
            category: PreflightCategory::ExternalBinary,
            required_for: "RAM Capture (Linux, optional)".into(),
            available: lime,
            detail: if lime_in_kit {
                "LiME modules in ./tools/lime/".into()
            } else if lime {
                "LiME module path present".into()
            } else {
                "LiME not detected (optional; avml preferred)".into()
            },
            install_hint: Some("Copy pre-built .ko files to ./tools/lime/ on forensic USB".into()),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let winpmem = ["winpmem.exe", "winpmem", "WinPmem", "winpmem_v4", "winpmem_v4.exe"]
            .iter()
            .find_map(|n| crate::portable::resolve_tool(n));
        checks.push(if let Some(r) = winpmem {
            PreflightCheck {
                id: "winpmem".into(),
                name: "WinPmem (RAM capture)".into(),
                category: PreflightCategory::ExternalBinary,
                required_for: "RAM Capture (Windows)".into(),
                available: r.hash_verified != Some(false),
                detail: format!("Found via {} at {}", r.source, r.path),
                install_hint: Some("Copy winpmem.exe to ./tools/ on forensic USB".into()),
            }
        } else {
            check_external_tool(
                "winpmem",
                "winpmem",
                "WinPmem (RAM capture)",
                "RAM Capture (Windows)",
            )
        });
        if crate::portable::tool_available("avml") {
            checks.push(check_external_tool(
                "avml",
                "avml",
                "avml (RAM capture)",
                "RAM Capture (Windows, optional)",
            ));
        }
    }

    #[cfg(target_os = "macos")]
    {
        checks.push(PreflightCheck {
            id: "macos_volatile".into(),
            name: "macOS volatile sources".into(),
            category: PreflightCategory::ExternalBinary,
            required_for: "macOS volatile triage".into(),
            available: true,
            detail: "Raw RAM acquisition intentionally disabled; use volatile artifacts and triage sources".into(),
            install_hint: Some("See the Apple Volatile Data section in the RAM guide".into()),
        });
    }

    checks
}

/// Run all pre-flight checks synchronously (no network probe).
pub fn run_preflight() -> PreflightReport {
    let mut checks = vec![];

    push_pure(&mut checks, "disk_imaging", "Disk imaging (RAW/E01/AFF4)", "Disk Imaging");
    push_pure(&mut checks, "hashing", "Hashing (MD5/SHA1/SHA256/Blake3)", "Hash Verify / CoC");
    push_pure(&mut checks, "signing", "Ed25519 signature + QR", "Chain of Custody");
    push_pure(&mut checks, "pdf", "PDF report", "CoC export");
    push_pure(
        &mut checks,
        "hpa_dco",
        "HPA/DCO detection",
        "Disk Imaging (ATA pass-through)",
    );
    push_pure(
        &mut checks,
        "encryption",
        "Encryption detection",
        "Encryption tab",
    );
    push_pure(&mut checks, "carving", "Carving engine", "Archive analysis");
    push_pure(&mut checks, "evidence_id", "Evidence ID numbering", "Chain of Custody");

    checks.push(check_libpcap());

    #[cfg(target_os = "linux")]
    checks.push(PreflightCheck {
        id: "blkroset".into(),
        name: "BLKROSET (software write-blocker)".into(),
        category: PreflightCategory::SystemLibrary,
        required_for: "Write Blocker (Linux)".into(),
        available: true,
        detail: "libc BLKROSET ioctl linked at compile time".into(),
        install_hint: None,
    });

    checks.extend(check_ram_tools());

    checks.push(check_external_tool(
        "adb",
        "adb",
        "adb (Android triage)",
        "Mobile Triage (Android)",
    ));

    checks.push(check_external_tool(
        "idevice_id",
        "idevice_id",
        "idevice_id (iOS device list)",
        "Mobile Triage (iOS)",
    ));

    checks.push(check_external_tool(
        "idevicebackup2",
        "idevicebackup2",
        "idevicebackup2 (iOS backup)",
        "Mobile Triage (iOS backup)",
    ));

    checks.push(PreflightCheck {
        id: "cloud_api".into(),
        name: "Cloud API (reqwest)".into(),
        category: PreflightCategory::ExternalBinary,
        required_for: "Cloud Snapshot".into(),
        available: true,
        detail: "HTTP client built in; requires live internet at capture time".into(),
        install_hint: Some("Ensure outbound HTTPS to your cloud provider is allowed".into()),
    });

    let elevated = is_elevated();
    #[cfg(target_os = "linux")]
    {
        checks.push(PreflightCheck {
            id: "priv_root".into(),
            name: "root / CAP_SYS_RAWIO".into(),
            category: PreflightCategory::Privilege,
            required_for: "Write blocker, HPA/DCO, raw disk access".into(),
            available: elevated,
            detail: if elevated {
                "Running as root".into()
            } else {
                "Not root — BLKROSET and SG_IO may fail".into()
            },
            install_hint: Some("Run CollectionLoom with sudo for full disk access on Linux".into()),
        });
    }

    #[cfg(target_os = "windows")]
    {
        checks.push(PreflightCheck {
            id: "priv_admin".into(),
            name: "Administrator".into(),
            category: PreflightCategory::Privilege,
            required_for: "Software write-blocker, raw disk IOCTL".into(),
            available: elevated,
            detail: if elevated {
                "Elevated session detected".into()
            } else {
                "Not elevated — run as Administrator for write-blocker".into()
            },
            install_hint: Some("Right-click CollectionLoom → Run as administrator".into()),
        });
    }

    #[cfg(target_os = "macos")]
    {
        checks.push(PreflightCheck {
            id: "priv_root".into(),
            name: "Administrator / sudo".into(),
            category: PreflightCategory::Privilege,
            required_for: "Diskutil unmount, volatile triage helpers".into(),
            available: elevated,
            detail: if elevated {
                "Running as root".into()
            } else {
                "Standard user — some operations will prompt for sudo".into()
            },
            install_hint: Some("Use sudo where prompted; macOS RAM acquisition itself is not provided".into()),
        });
    }

    let missing_count = checks
        .iter()
        .filter(|c| {
            !c.available
                && c.category != PreflightCategory::Privilege
                && c.id != "lime"
                && c.id != "cloud_api"
        })
        .count() as u32;

    let warning_count = checks
        .iter()
        .filter(|c| !c.available && c.category == PreflightCategory::Privilege)
        .count() as u32;

    let portable = crate::portable::portable_status();

    let mut summary = if missing_count == 0 && warning_count == 0 {
        if portable.portable_mode {
            "Portable kit ready — all tools detected; privileges OK.".into()
        } else {
            "All required external tools detected; privileges OK.".into()
        }
    } else if missing_count > 0 {
        if portable.portable_mode {
            format!(
                "{missing_count} tool(s) missing from ./tools/ — copy binaries to forensic USB before field use."
            )
        } else if portable.bundled_tools_available {
            format!(
                "{missing_count} tool(s) missing — rebuild with npm run download-tools or add to ./tools/."
            )
        } else {
            format!(
                "{missing_count} optional/required tool(s) missing — run npm run download-tools before build."
            )
        }
    } else {
        format!("{warning_count} privilege warning(s) — some features need elevation.")
    };

    if !portable.portable_mode && missing_count == 0 {
        if portable.bundled_tools_available {
            summary.push_str(" External tools are embedded in the app bundle.");
        } else {
            summary.push_str(" Tip: npm run download-tools embeds RAM/mobile tools at build time.");
        }
    }

    PreflightReport {
        platform: platform_label().into(),
        checked_at: Utc::now().to_rfc3339(),
        checks,
        missing_count,
        warning_count,
        summary,
        portable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_report_has_checks() {
        let r = run_preflight();
        assert!(!r.checks.is_empty());
        assert!(r.checks.iter().any(|c| c.category == PreflightCategory::PureRust));
    }
}
