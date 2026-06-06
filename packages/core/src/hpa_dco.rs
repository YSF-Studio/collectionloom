//! HPA / DCO detection via ATA IDENTIFY DEVICE and native-max queries.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HpaDcoReport {
    pub device: String,
    pub supported: bool,
    pub hpa_detected: bool,
    pub dco_detected: bool,
    pub identify_max_lba: Option<u64>,
    pub native_max_lba: Option<u64>,
    pub dco_max_lba: Option<u64>,
    pub hidden_sectors: Option<u64>,
    pub model: String,
    pub notes: String,
}

pub fn detect(device: &str) -> Result<HpaDcoReport, String> {
    let device = device.trim();
    if device.is_empty() {
        return Err("No device specified".into());
    }
    if device.contains("nvme") {
        return Ok(HpaDcoReport {
            device: device.to_string(),
            supported: false,
            hpa_detected: false,
            dco_detected: false,
            identify_max_lba: None,
            native_max_lba: None,
            dco_max_lba: None,
            hidden_sectors: None,
            model: String::new(),
            notes: "NVMe devices use a different command set; HPA/DCO ATA checks not applicable.".into(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        return detect_linux(device);
    }
    #[cfg(target_os = "windows")]
    {
        return detect_windows(device);
    }
    #[cfg(target_os = "macos")]
    {
        return detect_macos(device);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("HPA/DCO detection is not supported on this platform".into())
    }
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn word_u16(data: &[u8; 512], word: usize) -> u16 {
    let off = word * 2;
    u16::from_le_bytes([data[off], data[off + 1]])
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn identify_model(data: &[u8; 512]) -> String {
    let mut s = String::new();
    for w in (27..=46).step_by(1) {
        let v = word_u16(data, w);
        s.push(((v >> 8) & 0xff) as u8 as char);
        s.push((v & 0xff) as u8 as char);
    }
    s.trim().to_string()
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn identify_max_lba48(data: &[u8; 512]) -> u64 {
    let mut lba: u64 = 0;
    for w in 100..=103 {
        lba |= (word_u16(data, w) as u64) << ((w - 100) * 16);
    }
    lba
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn build_report(
    device: &str,
    model: String,
    identify_max: Option<u64>,
    native_max: Option<u64>,
    dco_max: Option<u64>,
    notes: &str,
) -> HpaDcoReport {
    let (hpa, hidden) = match (identify_max, native_max) {
        (Some(id), Some(native)) if id > native => (true, Some(id - native)),
        _ => (false, None),
    };
    let dco_detected = match (identify_max, dco_max) {
        (Some(id), Some(dco)) if dco != id => true,
        _ => false,
    };
    HpaDcoReport {
        device: device.to_string(),
        supported: true,
        hpa_detected: hpa,
        dco_detected,
        identify_max_lba: identify_max,
        native_max_lba: native_max,
        dco_max_lba: dco_max,
        hidden_sectors: hidden,
        model,
        notes: notes.to_string(),
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;

    const HDIO_DRIVE_CMD: libc::c_ulong = 0x031f;

    #[repr(C)]
    struct HdDriveCmd {
        command: u8,
        in_count: u8,
        out_count: u8,
        reserved: [u8; 5],
        data: [u8; 512],
    }

    fn ata_cmd(fd: i32, command: u8, out_sectors: u8) -> Result<[u8; 512], String> {
        let mut hdr = HdDriveCmd {
            command,
            in_count: 0,
            out_count: out_sectors,
            reserved: [0; 5],
            data: [0; 512],
        };
        let rc = unsafe { libc::ioctl(fd, HDIO_DRIVE_CMD as _, &mut hdr) };
        if rc < 0 {
            return Err(format!(
                "ATA command 0x{command:02X} failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(hdr.data)
    }

    fn native_max_lba(data: &[u8; 512]) -> Option<u64> {
        // READ NATIVE MAX ADDRESS EXT (0x27) returns max LBA in first 8 bytes (LE).
        let lba = u64::from_le_bytes(data[0..8].try_into().ok()?);
        if lba > 0 {
            Some(lba + 1)
        } else {
            None
        }
    }

    fn dco_identify_max(data: &[u8; 512]) -> Option<u64> {
        // DEVICE CONFIGURATION IDENTIFY populates same layout as IDENTIFY for max sector.
        let lba = identify_max_lba48(data);
        if lba > 0 {
            Some(lba)
        } else {
            None
        }
    }

    pub fn detect_linux(device: &str) -> Result<HpaDcoReport, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)
            .map_err(|e| format!("Cannot open {device}: {e}"))?;
        let fd = file.as_raw_fd();

        let identify = ata_cmd(fd, 0xEC, 1)?;
        let model = identify_model(&identify);
        let id_max = {
            let lba = identify_max_lba48(&identify);
            if lba > 0 { Some(lba) } else { None }
        };

        let native = ata_cmd(fd, 0x27, 1)
            .ok()
            .and_then(|d| native_max_lba(&d))
            .or_else(|| ata_cmd(fd, 0xF8, 1).ok().and_then(|d| {
                let lba = u64::from_le_bytes(d[0..8].try_into().ok()?);
                if lba > 0 { Some(lba + 1) } else { None }
            }));

        let dco = ata_cmd(fd, 0xB1, 1).ok().and_then(|d| dco_identify_max(&d));

        Ok(build_report(
            device,
            model,
            id_max,
            native,
            dco,
            "ATA IDENTIFY + READ NATIVE MAX + DCO IDENTIFY via HDIO_DRIVE_CMD",
        ))
    }
}

#[cfg(target_os = "linux")]
use platform::detect_linux;

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    // winapi 0.3 does not export these ATA pass-through symbols; define from Windows SDK.
    const IOCTL_DISK_BASE: DWORD = 0x0000_0007;
    const METHOD_BUFFERED: DWORD = 0;
    const FILE_READ_ACCESS: DWORD = 0x0001;
    const FILE_WRITE_ACCESS: DWORD = 0x0002;
    const ATA_FLAGS_DATA_IN: u8 = 0x01;

    const fn ctl_code(device_type: DWORD, function: DWORD, method: DWORD, access: DWORD) -> DWORD {
        (device_type << 16) | (access << 14) | (function << 2) | method
    }

    const IOCTL_ATA_PASS_THROUGH_DIRECT: DWORD =
        ctl_code(IOCTL_DISK_BASE, 0x001b, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    #[repr(C)]
    struct AtaPassThroughDirect {
        length: u16,
        ata_flags: u8,
        path_id: u8,
        data_transfer_length: u32,
        time_out_value: u32,
        reserved: u32,
        data_buffer: u64,
        previous: [u8; 8],
        current: [u8; 8],
    }

    fn open_device(path: &str) -> Result<winapi::um::winnt::HANDLE, String> {
        let win_path = crate::block_device::normalize_windows_path(path);
        let wide: Vec<u16> = OsStr::new(&win_path)
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
            return Err(format!("Cannot open {win_path}"));
        }
        Ok(handle)
    }

    fn ata_passthrough(handle: winapi::um::winnt::HANDLE, command: u8) -> Result<[u8; 512], String> {
        let mut buffer = [0u8; 512];
        let mut apt = AtaPassThroughDirect {
            length: std::mem::size_of::<AtaPassThroughDirect>() as u16,
            ata_flags: ATA_FLAGS_DATA_IN,
            path_id: 0,
            data_transfer_length: 512,
            time_out_value: 10,
            reserved: 0,
            data_buffer: buffer.as_mut_ptr() as u64,
            previous: [0; 8],
            current: [0; 8],
        };
        apt.current[6] = command;
        apt.current[9] = 0x08; // PIO data-in, DRQ within 8 sectors

        let mut bytes: DWORD = 0;
        let ok = unsafe {
            DeviceIoControl(
                handle,
                IOCTL_ATA_PASS_THROUGH_DIRECT,
                &mut apt as *mut _ as *mut _,
                std::mem::size_of::<AtaPassThroughDirect>() as DWORD,
                &mut apt as *mut _ as *mut _,
                std::mem::size_of::<AtaPassThroughDirect>() as DWORD,
                &mut bytes,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err("IOCTL_ATA_PASS_THROUGH_DIRECT failed".into());
        }
        Ok(buffer)
    }

    pub fn detect_windows(device: &str) -> Result<HpaDcoReport, String> {
        let handle = open_device(device)?;
        let identify = ata_passthrough(handle, 0xEC)?;
        let model = identify_model(&identify);
        let id_max = {
            let lba = identify_max_lba48(&identify);
            if lba > 0 { Some(lba) } else { None }
        };
        let native = ata_passthrough(handle, 0x27)
            .ok()
            .and_then(|d| {
                let lba = u64::from_le_bytes(d[0..8].try_into().ok()?);
                if lba > 0 { Some(lba + 1) } else { None }
            });
        let dco = ata_passthrough(handle, 0xB1)
            .ok()
            .and_then(|d| {
                let lba = identify_max_lba48(&d);
                if lba > 0 { Some(lba) } else { None }
            });
        unsafe {
            CloseHandle(handle);
        }
        Ok(build_report(
            device,
            model,
            id_max,
            native,
            dco,
            "ATA IDENTIFY via IOCTL_ATA_PASS_THROUGH_DIRECT",
        ))
    }
}

#[cfg(target_os = "windows")]
use platform::detect_windows;

#[cfg(target_os = "macos")]
fn detect_macos(device: &str) -> Result<HpaDcoReport, String> {
    // macOS blocks raw ATA on many external enclosures; report honestly.
    Ok(HpaDcoReport {
        device: device.to_string(),
        supported: false,
        hpa_detected: false,
        dco_detected: false,
        identify_max_lba: None,
        native_max_lba: None,
        dco_max_lba: None,
        hidden_sectors: None,
        model: String::new(),
        notes: "macOS does not expose ATA pass-through for most USB/SATA bridges. Use Linux or Windows for HPA/DCO checks.".into(),
    })
}
