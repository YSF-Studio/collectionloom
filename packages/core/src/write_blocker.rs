
use serde::Serialize;

/// Structured write-blocker status for UI and titlebar badge.
#[derive(Debug, Clone, Serialize)]
pub struct WriteBlockerStatus {
    pub active: bool,
    pub enabled: bool,
    pub method: String,
    pub confidence: String,
    pub hardware: bool,
    pub notes: String,
}

impl WriteBlockerStatus {
    fn inactive(method: &str, notes: &str) -> Self {
        Self {
            active: false,
            enabled: false,
            method: method.into(),
            confidence: "none".into(),
            hardware: false,
            notes: notes.into(),
        }
    }
}

/// Check write-blocker status (structured).
pub fn check_write_blocker_status(device: &str) -> WriteBlockerStatus {
    if device.is_empty() {
        return WriteBlockerStatus::inactive("none", "No device selected");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("lsblk").args(["-o", "NAME,RO"]).arg(device).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let ro = stdout.lines().any(|l| l.trim().ends_with('1'));
            if ro {
                return WriteBlockerStatus {
                    active: true,
                    enabled: true,
                    method: "BLKROSET (kernel read-only)".into(),
                    confidence: "high".into(),
                    hardware: false,
                    notes: "Software write-block via block device read-only flag.".into(),
                };
            }
        }
        if detect_hardware_blocker_linux() {
            return WriteBlockerStatus {
                active: true,
                enabled: true,
                method: "Hardware write-blocker (USB)".into(),
                confidence: "high".into(),
                hardware: true,
                notes: "Tableau/WiebeTech-class hardware blocker detected.".into(),
            };
        }
        WriteBlockerStatus::inactive(
            "BLKROSET",
            "Enable software blocker or attach hardware write-blocker before imaging.",
        )
    }

    #[cfg(target_os = "macos")]
    {
        macos_status(device)
    }

    #[cfg(target_os = "windows")]
    {
        WriteBlockerStatus::inactive(
            "DeviceIoControl",
            "Use hardware write-blocker or run CollectionLoom as Administrator.",
        )
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        WriteBlockerStatus::inactive("unsupported", "Platform not supported")
    }
}

/// Legacy bool API — true when blocker is considered active.
pub fn check_write_blocker(device: &str) -> bool {
    check_write_blocker_status(device).active
}

#[cfg(target_os = "macos")]
fn macos_status(device: &str) -> WriteBlockerStatus {
    use std::process::Command;

    if detect_hardware_blocker_macos() {
        return WriteBlockerStatus {
            active: true,
            enabled: true,
            method: "Hardware write-blocker (USB)".into(),
            confidence: "high".into(),
            hardware: true,
            notes: "External hardware write-blocker detected via USB profile.".into(),
        };
    }

    let disk = normalize_macos_disk(device);
    if let Ok(output) = Command::new("diskutil").args(["info", &disk]).output() {
        let info = String::from_utf8_lossy(&output.stdout);
        let read_only_media = info.contains("Read-Only Media:           Yes")
            || info.contains("Read-Only Media: Yes");
        let writable_no = info.contains("Writable:                  No")
            || info.contains("Writable: No");
        let mounted = info.contains("Mounted:                   Yes")
            || info.contains("Mounted: Yes");

        if read_only_media || writable_no {
            return WriteBlockerStatus {
                active: true,
                enabled: true,
                method: "diskutil read-only / unmounted".into(),
                confidence: if read_only_media { "high" } else { "medium" }.into(),
                hardware: false,
                notes: if mounted {
                    "Volume mounted read-only. Raw imaging via /dev/rdisk* is still recommended.".into()
                } else {
                    "Disk unmounted or read-only — safe for acquisition.".into()
                },
            };
        }
    }

    WriteBlockerStatus::inactive(
        "diskutil unmount + hardware",
        "Connect hardware write-blocker, or click Enable to unmount disk volumes before imaging.",
    )
}

#[cfg(target_os = "macos")]
fn normalize_macos_disk(device: &str) -> String {
    let d = device.trim();
    if d.starts_with("/dev/rdisk") {
        d.replacen("rdisk", "disk", 1)
    } else if d.starts_with("/dev/") {
        d.to_string()
    } else {
        format!("/dev/{d}")
    }
}

#[cfg(target_os = "macos")]
fn detect_hardware_blocker_macos() -> bool {
    use std::process::Command;
    let Ok(output) = Command::new("system_profiler").args(["SPUSBDataType"]).output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    ["tableau", "wiebetech", "writeblock", "forensic", "logicube", "deepSpar"]
        .iter()
        .any(|k| text.contains(k))
}

#[cfg(target_os = "linux")]
fn detect_hardware_blocker_linux() -> bool {
    use std::process::Command;
    let Ok(output) = Command::new("lsusb").output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    ["tableau", "wiebetech", "logicube"].iter().any(|k| text.contains(k))
}

/// Enable write blocker for a device.
///
/// **Activation flow:**
/// 1. **Hardware (recommended):** Connect USB write-blocker → auto-detected → badge turns green.
/// 2. **Software Linux:** BLKROSET ioctl sets kernel read-only on block device.
/// 3. **Software macOS:** Unmounts all volumes on the disk (`diskutil unmountDisk force`) so
///    mounted filesystems cannot be written during triage. Raw imaging uses `/dev/rdiskN` directly.
pub fn enable_write_blocker(device: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open {device}: {e}"))?;
        let ro: i32 = 1;
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000125D, &ro) };
        if ret != 0 {
            return Err(format!("BLKROSET failed on {device} (errno: {ret})"));
        }
        let test = std::fs::OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(false)
            .open(device);
        if test.is_ok() {
            return Err(format!(
                "Write blocker MAY NOT be active on {device} — use hardware blocker"
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let disk = normalize_macos_disk(device);
        // Unmount all volumes — primary software protection on macOS
        let status = Command::new("diskutil")
            .args(["unmountDisk", "force", &disk])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "Could not unmount {disk}. Eject other apps using the disk, or use a hardware write-blocker."
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Err("Windows write blocker requires administrator elevation — use hardware write-blocker".into())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported platform".into())
    }
}

/// Disable write blocker (remount / clear BLKROSET).
pub fn disable_write_blocker(device: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open {device}: {e}"))?;
        let rw: i32 = 0;
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000125D, &rw) };
        if ret != 0 {
            return Err(format!("BLKROSET clear failed on {device}"));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let disk = normalize_macos_disk(device);
        Command::new("diskutil")
            .args(["mountDisk", &disk])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".into())
    }
}
