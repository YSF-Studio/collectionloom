
use once_cell::sync::Lazy;
use serde::Serialize;
use ts_rs::TS;
use std::collections::HashSet;
use std::sync::Mutex;

/// Devices where software write-block was explicitly enabled this session.
static SOFTWARE_BLOCKED: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

fn mark_software(device: &str) {
    if let Ok(mut set) = SOFTWARE_BLOCKED.lock() {
        set.insert(normalize_device_key(device));
    }
}

fn unmark_software(device: &str) {
    if let Ok(mut set) = SOFTWARE_BLOCKED.lock() {
        set.remove(&normalize_device_key(device));
    }
}

fn is_marked_software(device: &str) -> bool {
    SOFTWARE_BLOCKED
        .lock()
        .map(|s| s.contains(&normalize_device_key(device)))
        .unwrap_or(false)
}

fn normalize_device_key(device: &str) -> String {
    device.trim().to_lowercase()
}

/// Structured write-blocker status for UI and titlebar badge.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/WriteBlockerStatus.ts")]
pub struct WriteBlockerStatus {
    pub active: bool,
    pub enabled: bool,
    pub method: String,
    pub confidence: String,
    pub hardware: bool,
    pub software: bool,
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
            software: false,
            notes: notes.into(),
        }
    }

    fn software(method: &str, notes: &str, confidence: &str) -> Self {
        Self {
            active: true,
            enabled: true,
            method: method.into(),
            confidence: confidence.into(),
            hardware: false,
            software: true,
            notes: notes.into(),
        }
    }
}

fn validate_device(device: &str) -> Result<(), String> {
    let d = device.trim();
    if d.is_empty() {
        return Err("No device selected".into());
    }
    if d.chars().any(|c| matches!(c, ';' | '&' | '|' | '$' | '`' | '\n' | '\r' | '\0')) {
        return Err("Invalid device path: illegal characters".into());
    }
    #[cfg(unix)]
    {
        if !d.starts_with("/dev/") && !d.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(format!("Invalid device path: {d}"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        let lower = d.to_lowercase();
        if !(lower.starts_with("\\\\.\\physicaldrive")
            || lower.starts_with("physicaldrive")
            || d.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(format!("Invalid device path: {d}"));
        }
    }
    Ok(())
}

/// Check write-blocker status (structured).
pub fn check_write_blocker_status(device: &str) -> WriteBlockerStatus {
    if validate_device(device).is_err() {
        return WriteBlockerStatus::inactive("none", "Invalid device path");
    }

    if detect_hardware_blocker() {
        return WriteBlockerStatus {
            active: true,
            enabled: true,
            method: "Hardware write-blocker (USB)".into(),
            confidence: "high".into(),
            hardware: true,
            software: false,
            notes: "External hardware write-blocker detected.".into(),
        };
    }

    #[cfg(target_os = "linux")]
    {
        if linux_is_readonly(device) {
            return WriteBlockerStatus {
                active: true,
                enabled: true,
                method: "BLKROSET (kernel read-only)".into(),
                confidence: "high".into(),
                hardware: false,
                software: is_marked_software(device),
                notes: "Software write-block active via kernel read-only flag.".into(),
            };
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(status) = macos_disk_status(device) {
            return status;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(status) = windows_disk_status(device) {
            return status;
        }
    }

    if is_marked_software(device) {
        return WriteBlockerStatus::software(
            "Software (session)",
            "Software write-block enabled — re-check after reconnecting device.",
            "medium",
        );
    }

    #[cfg(target_os = "linux")]
    {
        return WriteBlockerStatus::inactive(
            "BLKROSET",
            "Click Enable Software Write-Blocker or attach hardware blocker before imaging.",
        );
    }
    #[cfg(target_os = "macos")]
    {
        return WriteBlockerStatus::inactive(
            "diskutil unmount",
            "Click Enable Software Write-Blocker to unmount volumes, or use hardware blocker.",
        );
    }
    #[cfg(target_os = "windows")]
    {
        return WriteBlockerStatus::inactive(
            "IOCTL read-only",
            "Click Enable Software Write-Blocker (requires Administrator). Hardware blocker recommended.",
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        WriteBlockerStatus::inactive("unsupported", "Platform not supported")
    }
}

pub fn check_write_blocker(device: &str) -> bool {
    check_write_blocker_status(device).active
}

fn detect_hardware_blocker() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("lsusb").output() {
            let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
            return ["tableau", "wiebetech", "logicube", "writeblock"]
                .iter()
                .any(|k| text.contains(k));
        }
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let Ok(output) = Command::new("system_profiler").args(["SPUSBDataType"]).output() else {
            return false;
        };
        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
        return ["tableau", "wiebetech", "writeblock", "forensic", "logicube", "deepspar"]
            .iter()
            .any(|k| text.contains(k));
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-PnpDevice -PresentOnly | Where-Object { $_.FriendlyName -match 'Tableau|WiebeTech|WriteBlock|Forensic' } | Select-Object -First 1",
            ])
            .output()
        {
            return !output.stdout.is_empty();
        }
        false
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

#[cfg(target_os = "linux")]
fn linux_is_readonly(device: &str) -> bool {
    use std::process::Command;
    if let Ok(output) = Command::new("lsblk").args(["-o", "NAME,RO"]).arg(device).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.lines().any(|l| l.trim().ends_with('1'));
    }
    if let Ok(output) = Command::new("blockdev").args(["--getro", device]).output() {
        return String::from_utf8_lossy(&output.stdout).trim() == "1";
    }
    false
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
fn macos_disk_status(device: &str) -> Option<WriteBlockerStatus> {
    use std::process::Command;
    let disk = normalize_macos_disk(device);
    if is_marked_software(device) {
        return Some(WriteBlockerStatus::software(
            "diskutil unmount (software)",
            "Volumes unmounted — safe for raw imaging via /dev/rdiskN.",
            "high",
        ));
    }
    if let Ok(output) = Command::new("diskutil").args(["info", &disk]).output() {
        let info = String::from_utf8_lossy(&output.stdout);
        let read_only = info.contains("Read-Only Media:           Yes")
            || info.contains("Read-Only Media: Yes");
        let writable_no = info.contains("Writable:                  No")
            || info.contains("Writable: No");
        let mounted = info.contains("Mounted:                   Yes") || info.contains("Mounted: Yes");
        if read_only || writable_no {
            return Some(WriteBlockerStatus {
                active: true,
                enabled: true,
                method: "diskutil read-only / unmounted".into(),
                confidence: if read_only { "high" } else { "medium" }.into(),
                hardware: false,
                software: true,
                notes: if mounted {
                    "Volume read-only — prefer /dev/rdiskN for imaging.".into()
                } else {
                    "Disk unmounted — ready for acquisition.".into()
                },
            });
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_disk_status(device: &str) -> Option<WriteBlockerStatus> {
    if is_marked_software(device) {
        return Some(WriteBlockerStatus::software(
            "IOCTL read-only (software)",
            "Disk set to read-only via Windows IOCTL — run as Administrator.",
            "high",
        ));
    }
    if windows_is_readonly(device) {
        return Some(WriteBlockerStatus::software(
            "IOCTL read-only",
            "Physical disk is read-only.",
            "high",
        ));
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_is_readonly(device: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::winioctl::IOCTL_DISK_GET_DISK_ATTRIBUTES;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ};

    const DISK_ATTRIBUTE_READ_ONLY: u64 = 0x0000_0000_0000_0002;

    let path = crate::block_device::normalize_windows_path(device);
    let wide: Vec<u16> = OsStr::new(&path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return false;
    }

    #[repr(C)]
    struct DiskAttributes {
        version: u32,
        reserved1: u32,
        attributes: u64,
    }

    let mut attrs = DiskAttributes {
        version: 1,
        reserved1: 0,
        attributes: 0,
    };
    let mut bytes: DWORD = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_DISK_ATTRIBUTES,
            ptr::null_mut(),
            0,
            &mut attrs as *mut _ as *mut _,
            std::mem::size_of::<DiskAttributes>() as DWORD,
            &mut bytes,
            ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }
    ok != 0 && (attrs.attributes & DISK_ATTRIBUTE_READ_ONLY) != 0
}

/// Enable software write blocker — one-click on all supported platforms.
pub fn enable_write_blocker(device: &str) -> Result<WriteBlockerStatus, String> {
    validate_device(device)?;

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open {device}: {e}"))?;
        let ro: i32 = 1;
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000_125D, &ro) };
        if ret != 0 {
            return Err(format!("BLKROSET failed on {device} — try sudo or hardware blocker"));
        }
        mark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let disk = normalize_macos_disk(device);
        let status = Command::new("diskutil")
            .args(["unmountDisk", "force", &disk])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "Could not unmount {disk}. Close apps using the disk, then retry Enable."
            ));
        }
        mark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(target_os = "windows")]
    {
        windows_enable_readonly(device)?;
        mark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported platform".into())
    }
}

#[cfg(target_os = "windows")]
fn windows_enable_readonly(device: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::winioctl::IOCTL_DISK_SET_DISK_ATTRIBUTES;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    const DISK_ATTRIBUTE_READ_ONLY: u64 = 0x0000_0000_0000_0002;

    let path = crate::block_device::normalize_windows_path(device);
    let wide: Vec<u16> = OsStr::new(&path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(
            "Cannot open physical drive — run CollectionLoom as Administrator".into(),
        );
    }

    #[repr(C)]
    struct SetDiskAttributes {
        version: u32,
        reserved1: u16,
        persist: u8,
        reserved2: u8,
        attributes: u64,
        attributes_mask: u64,
    }

    let mut input = SetDiskAttributes {
        version: 1,
        reserved1: 0,
        persist: 1,
        reserved2: 0,
        attributes: DISK_ATTRIBUTE_READ_ONLY,
        attributes_mask: DISK_ATTRIBUTE_READ_ONLY,
    };
    let mut bytes: DWORD = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_SET_DISK_ATTRIBUTES,
            &mut input as *mut _ as *mut _,
            std::mem::size_of::<SetDiskAttributes>() as DWORD,
            ptr::null_mut(),
            0,
            &mut bytes,
            ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }
    if ok == 0 {
        return Err("IOCTL_DISK_SET_DISK_ATTRIBUTES failed — run as Administrator".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_disable_readonly(device: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::winioctl::IOCTL_DISK_SET_DISK_ATTRIBUTES;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    const DISK_ATTRIBUTE_READ_ONLY: u64 = 0x0000_0000_0000_0002;

    let path = crate::block_device::normalize_windows_path(device);
    let wide: Vec<u16> = OsStr::new(&path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null_mut(),
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err("Cannot open physical drive".into());
    }

    #[repr(C)]
    struct SetDiskAttributes {
        version: u32,
        reserved1: u16,
        persist: u8,
        reserved2: u8,
        attributes: u64,
        attributes_mask: u64,
    }

    let mut input = SetDiskAttributes {
        version: 1,
        reserved1: 0,
        persist: 1,
        reserved2: 0,
        attributes: 0,
        attributes_mask: DISK_ATTRIBUTE_READ_ONLY,
    };
    let mut bytes: DWORD = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_SET_DISK_ATTRIBUTES,
            &mut input as *mut _ as *mut _,
            std::mem::size_of::<SetDiskAttributes>() as DWORD,
            ptr::null_mut(),
            0,
            &mut bytes,
            ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }
    if ok == 0 {
        return Err("Failed to clear read-only flag".into());
    }
    Ok(())
}

/// Disable software write blocker.
pub fn disable_write_blocker(device: &str) -> Result<WriteBlockerStatus, String> {
    validate_device(device)?;

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open {device}: {e}"))?;
        let rw: i32 = 0;
        let ret = unsafe { libc::ioctl(file.as_raw_fd(), 0x0000_125D, &rw) };
        if ret != 0 {
            return Err(format!("BLKROSET clear failed on {device}"));
        }
        unmark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let disk = normalize_macos_disk(device);
        Command::new("diskutil")
            .args(["mountDisk", &disk])
            .status()
            .map_err(|e| e.to_string())?;
        unmark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(target_os = "windows")]
    {
        windows_disable_readonly(device)?;
        unmark_software(device);
        return Ok(check_write_blocker_status(device));
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported platform".into())
    }
}
