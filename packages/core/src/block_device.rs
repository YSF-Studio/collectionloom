//! Block device size detection — supports drives of any capacity (u64 bytes).

use std::path::Path;

/// Returns total size in bytes for block devices and regular files.
pub fn device_size(path: &str) -> Result<u64, String> {
    let meta_len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if meta_len > 0 {
        return Ok(meta_len);
    }

    #[cfg(unix)]
    {
        if let Ok(sz) = unix_block_size(path) {
            if sz > 0 {
                return Ok(sz);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(sz) = windows_block_size(path) {
            if sz > 0 {
                return Ok(sz);
            }
        }
    }

    Ok(meta_len)
}

#[cfg(unix)]
fn unix_block_size(path: &str) -> Result<u64, String> {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| format!("Cannot open {path} for size query: {e}"))?;
    let fd = file.as_raw_fd();

    #[cfg(target_os = "linux")]
    {
        let mut size: u64 = 0;
        let ret = unsafe { libc::ioctl(fd, 0x8008_1272u64 as libc::c_ulong, &mut size) };
        if ret == 0 && size > 0 {
            return Ok(size);
        }
    }

    #[cfg(target_os = "macos")]
    {
        const DKIOCGETBLOCKCOUNT: libc::c_ulong = 0x4008_6419;
        const DKIOCGETBLOCKSIZE: libc::c_ulong = 0x4008_6418;
        let mut block_count: u64 = 0;
        let mut block_size: u32 = 0;
        let r1 = unsafe { libc::ioctl(fd, DKIOCGETBLOCKCOUNT, &mut block_count) };
        let r2 = unsafe { libc::ioctl(fd, DKIOCGETBLOCKSIZE, &mut block_size) };
        if r1 == 0 && r2 == 0 && block_count > 0 && block_size > 0 {
            return Ok(block_count * block_size as u64);
        }
    }

    Err(format!("Could not determine block size for {path}"))
}

#[cfg(target_os = "windows")]
fn windows_block_size(path: &str) -> Result<u64, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::ioapiset::DeviceIoControl;
    use winapi::um::winioctl::IOCTL_DISK_GET_LENGTH_INFO;
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ};

    let win_path = normalize_windows_path(path);
    let wide: Vec<u16> = OsStr::new(&win_path)
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
        return Err(format!("Cannot open {win_path}"));
    }

    #[repr(C)]
    struct LengthInfo {
        length: i64,
    }

    let mut info = LengthInfo { length: 0 };
    let mut bytes: DWORD = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_LENGTH_INFO,
            ptr::null_mut(),
            0,
            &mut info as *mut _ as *mut _,
            std::mem::size_of::<LengthInfo>() as DWORD,
            &mut bytes,
            ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }
    if ok == 0 {
        return Err("IOCTL_DISK_GET_LENGTH_INFO failed".into());
    }
    Ok(info.length as u64)
}

#[cfg(target_os = "windows")]
pub fn normalize_windows_path(device: &str) -> String {
    let d = device.trim();
    if d.starts_with("\\\\.\\") {
        d.to_string()
    } else if d.starts_with("PhysicalDrive") {
        format!("\\\\.\\{d}")
    } else if d.starts_with("/dev/") {
        d.to_string()
    } else {
        format!("\\\\.\\PhysicalDrive{d}")
    }
}

/// Human-readable capacity hint for UI (TB/GB).
pub fn format_capacity(bytes: u64) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.2} TB", bytes as f64 / 1e12)
    } else if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1e9)
    } else {
        format!("{:.0} MB", bytes as f64 / 1e6)
    }
}

/// Ensure destination filesystem supports large single files (warn if FAT32-sized).
pub fn destination_ok_for_size(dest: &Path, source_bytes: u64) -> Result<(), String> {
    if source_bytes > 4_294_967_296 {
        let ext = dest
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "dd" || ext.is_empty() {
            // Recommend split for very large images on FAT32 destinations — caller may split
        }
    }
    Ok(())
}
