use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Instant;
use sha2::Digest;

use crate::imaging_format::ImageFormat;
use crate::progress::ImagingSummary;
use crate::bad_sector::{read_resilient, BadSectorLog, DEFAULT_SECTOR_SIZE};

/// Bytes hashed for pre/post source integrity (sectors 0–99, 51200 bytes).
pub const SOURCE_INTEGRITY_BYTES: u64 = 51200;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AcquisitionState {
    Idle,
    PreTriage,
    PreTriageDone,
    AwaitingDecision,
    CapturingRam,
    Imaging,
    Verifying,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub size_bytes: u64,
    pub sector_size: u64,
    pub is_ssd: bool,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionInfo {
    pub device: String,
    pub size_bytes: u64,
    pub file_system: String,
}

impl DiskInfo {
    #[cfg(target_os = "linux")]
    pub fn list() -> Result<Vec<Self>, String> {
        use std::process::Command;
        let output = Command::new("lsblk")
            .args(["-J", "-o", "NAME,SIZE,MODEL,ROTA,TYPE,MOUNTPOINT,FSTYPE"])
            .output().map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        let mut disks = vec![];
        if let Some(devices) = json["blockdevices"].as_array() {
            for d in devices {
                if d["type"].as_str() == Some("disk") {
                    let size_str = d["size"].as_str().unwrap_or("0");
                    let size = parse_size(size_str);
                    disks.push(DiskInfo {
                        device: format!("/dev/{}", d["name"].as_str().unwrap_or("?")),
                        model: d["model"].as_str().unwrap_or("Unknown").to_string(),
                        size_bytes: size,
                        sector_size: 512,
                        is_ssd: d["rota"].as_str() == Some("0"),
                        partitions: vec![],
                    });
                }
            }
        }
        Ok(disks)
    }

    #[cfg(target_os = "macos")]
    pub fn list() -> Result<Vec<Self>, String> {
        use std::process::Command;
        // List all physical disks (internal + external). "external physical" omitted internal drives.
        let output = Command::new("diskutil")
            .args(["list", "physical"])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("diskutil list failed: {stderr}"));
        }
        let mut disks = parse_diskutil_list_text(&String::from_utf8_lossy(&output.stdout));
        if disks.is_empty() {
            let fallback = Command::new("diskutil")
                .arg("list")
                .output()
                .map_err(|e| e.to_string())?;
            disks = parse_diskutil_list_text(&String::from_utf8_lossy(&fallback.stdout));
        }
        Ok(disks)
    }

    #[cfg(target_os = "windows")]
    pub fn list() -> Result<Vec<Self>, String> {
        use std::process::Command;
        let ps = r#"Get-CimInstance -ClassName Win32_DiskDrive | ForEach-Object { "{0}|{1}|{2}|{3}" -f $_.Index, ($_.Model -replace '\|','/'), $_.Size, $_.MediaType }"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", ps])
            .output()
            .map_err(|e| format!("PowerShell disk query failed: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Win32_DiskDrive query failed: {stderr}"));
        }
        let mut disks = vec![];
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() < 3 {
                continue;
            }
            let index: u32 = parts[0].trim().parse().unwrap_or(999);
            let model = parts[1].trim().to_string();
            let size_bytes: u64 = parts[2].trim().parse().unwrap_or(0);
            let media = parts.get(3).unwrap_or(&"").trim().to_lowercase();
            let is_ssd = media.contains("ssd") || media.contains("nvme") || media.contains("solid");
            disks.push(DiskInfo {
                device: format!(r"\\.\PhysicalDrive{index}"),
                model: if model.is_empty() { "Unknown".into() } else { model },
                size_bytes,
                sector_size: 512,
                is_ssd,
                partitions: vec![],
            });
        }
        disks.sort_by_key(|d| d.device.clone());
        Ok(disks)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    pub fn list() -> Result<Vec<Self>, String> {
        Ok(vec![])
    }
}

#[cfg(target_os = "macos")]
fn parse_diskutil_list_text(stdout: &str) -> Vec<DiskInfo> {
    let mut disks = vec![];
    let mut current: Option<(String, String)> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("/dev/disk") {
            continue;
        }
        // Whole disks are /dev/diskN — skip partition paths like /dev/disk0s1.
        let device_part = trimmed.split_whitespace().next().unwrap_or("");
        if !is_macos_whole_disk(device_part) {
            continue;
        }
        if let Some((device, size_hint)) = current.take() {
            push_macos_disk(&mut disks, device, size_hint);
        }
        let head = trimmed.split(':').next().unwrap_or(trimmed).trim();
        let mut head_parts = head.split_whitespace();
        let device = head_parts.next().unwrap_or("").to_string();
        let hint = head_parts.collect::<Vec<_>>().join(" ");
        current = Some((device, hint));
    }
    if let Some((device, size_hint)) = current {
        push_macos_disk(&mut disks, device, size_hint);
    }
    disks
}

#[cfg(target_os = "macos")]
fn is_macos_whole_disk(device: &str) -> bool {
    let name = device.trim_start_matches("/dev/");
    if !name.starts_with("disk") {
        return false;
    }
    let suffix = &name[4..];
    !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
}

#[cfg(target_os = "macos")]
fn push_macos_disk(disks: &mut Vec<DiskInfo>, device: String, size_hint: String) {
    if let Ok(info) = disk_info_macos(&device) {
        disks.push(info);
    } else {
        disks.push(DiskInfo {
            device,
            model: size_hint,
            size_bytes: 0,
            sector_size: 512,
            is_ssd: true,
            partitions: vec![],
        });
    }
}

#[cfg(target_os = "macos")]
fn disk_info_macos(device: &str) -> Result<DiskInfo, String> {
    use std::process::Command;
    let output = Command::new("diskutil")
        .args(["info", device])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "diskutil info failed for {device}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut model = "Unknown".to_string();
    let mut size_bytes = 0u64;
    let mut is_ssd = true;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Device / Media Name:") {
            model = line
                .split(':')
                .nth(1)
                .unwrap_or("Unknown")
                .trim()
                .to_string();
        } else if line.starts_with("Disk Size:") {
            if let Some(open) = line.find('(') {
                if let Some(close) = line[open + 1..].find(" Bytes)") {
                    let num = line[open + 1..open + 1 + close].trim();
                    size_bytes = num.parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("Solid State:") {
            is_ssd = line.contains("Yes") || line.contains("yes");
        } else if line.starts_with("Protocol:") && line.to_ascii_lowercase().contains("usb") {
            is_ssd = true;
        }
    }
    Ok(DiskInfo {
        device: device.to_string(),
        model,
        size_bytes,
        sector_size: 512,
        is_ssd,
        partitions: vec![],
    })
}

#[cfg(target_os = "linux")]
fn parse_size(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    if s.ends_with("G") { s.trim_end_matches('G').parse::<f64>().unwrap_or(0.0) as u64 * 1_073_741_824 }
    else if s.ends_with("M") { s.trim_end_matches('M').parse::<f64>().unwrap_or(0.0) as u64 * 1_048_576 }
    else if s.ends_with("T") { s.trim_end_matches('T').parse::<f64>().unwrap_or(0.0) as u64 * 1_099_511_627_776 }
    else { s.parse().unwrap_or(0) }
}

fn hash_prefix(path: &Path, max_bytes: u64) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
    let mut remaining = max_bytes;
    while remaining > 0 {
        let to_read = buf.len().min(remaining as usize);
        let n = file.read(&mut buf[..to_read]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_prefix_from_reader(reader: &mut impl Read, max_bytes: u64) -> Result<String, String> {
    let mut hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
    let mut remaining = max_bytes;
    while remaining > 0 {
        let to_read = buf.len().min(remaining as usize);
        let n = reader.read(&mut buf[..to_read]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_split_raw_parts(
    dir: &Path,
    stem: &str,
    part_count: u32,
    expected: &str,
) -> Result<(), String> {
    let mut hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
    for part in 1..=part_count {
        let name = if part_count == 1 {
            stem.to_string()
        } else {
            format!("{stem}.{part:05}")
        };
        let path = dir.join(&name);
        let mut file = File::open(&path).map_err(|e| format!("Verify open {}: {e}", path.display()))?;
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        return Err(format!("Split verify failed: stream {expected} != parts {actual}"));
    }
    Ok(())
}

/// Stream disk to image file with progress, split, and verify
pub struct DiskImager {
    pub source: String,
    pub destination: PathBuf,
    pub split_size: Option<u64>,
    pub verify: bool,
    pub format: ImageFormat,
}

impl DiskImager {
    pub fn new(source: &str, dest: &Path) -> Self {
        Self {
            source: source.to_string(),
            destination: dest.to_path_buf(),
            split_size: None,
            verify: true,
            format: ImageFormat::Raw,
        }
    }

    pub fn run(&self, cancel_flag: &std::sync::atomic::AtomicBool) -> Result<String, String> {
        let source = crate::imaging_format::normalize_block_source(&self.source);
        let started = Instant::now();
        let mut bad_log = BadSectorLog::new();

        // Pre-imaging source integrity hash (first ~51200 bytes)
        let pre_source_hash = {
            let mut src = File::open(&source)
                .map_err(|e| format!("Cannot open source for integrity check: {e}"))?;
            hash_prefix_from_reader(&mut src, SOURCE_INTEGRITY_BYTES)?
        };

        let (hash, mut summary) = match self.format {
            ImageFormat::E01 => {
                let hash = crate::imaging_format::acquire_e01(
                    &source,
                    &self.destination,
                    cancel_flag,
                    &mut bad_log,
                )?;
                let duration = started.elapsed().as_secs_f64();
                let bytes = crate::block_device::device_size(&source).unwrap_or(0);
                let sectors = if bytes > 0 { bytes / 512 } else { 0 };
                (
                    hash.clone(),
                    ImagingSummary {
                        sha256: hash,
                        sectors_read: sectors,
                        avg_speed_bytes_per_sec: if duration > 0.0 { bytes as f64 / duration } else { 0.0 },
                        error_sectors: bad_log.error_sectors,
                        duration_secs: duration,
                        source_integrity_ok: true,
                        bytes_written: bytes,
                        bad_sectors_log: None,
                    },
                )
            }
            ImageFormat::Aff4 => {
                let hash = crate::imaging_format::acquire_aff4(
                    &source,
                    &self.destination,
                    self.split_size,
                    self.verify,
                    cancel_flag,
                    &mut bad_log,
                )?;
                let duration = started.elapsed().as_secs_f64();
                let bytes = crate::block_device::device_size(&source).unwrap_or(0);
                let sectors = if bytes > 0 { bytes / 512 } else { 0 };
                (
                    hash.clone(),
                    ImagingSummary {
                        sha256: hash,
                        sectors_read: sectors,
                        avg_speed_bytes_per_sec: if duration > 0.0 { bytes as f64 / duration } else { 0.0 },
                        error_sectors: bad_log.error_sectors,
                        duration_secs: duration,
                        source_integrity_ok: true,
                        bytes_written: bytes,
                        bad_sectors_log: None,
                    },
                )
            }
            ImageFormat::Raw => self.run_raw(&source, cancel_flag, started, pre_source_hash.clone(), &mut bad_log)?,
        };

        if bad_log.error_sectors > 0 {
            if let Ok(Some(log_path)) = bad_log.write_log_file(&self.destination) {
                summary.bad_sectors_log = Some(log_path.to_string_lossy().into_owned());
                summary.error_sectors = bad_log.error_sectors;
            }
        }

        // Post-imaging source integrity: re-read prefix from source
        let post_source_hash = {
            let mut src = File::open(&source)
                .map_err(|e| format!("Cannot re-read source for integrity check: {e}"))?;
            hash_prefix_from_reader(&mut src, SOURCE_INTEGRITY_BYTES)?
        };
        let source_integrity_ok = pre_source_hash == post_source_hash;

        let mut final_summary = summary;
        final_summary.source_integrity_ok = source_integrity_ok;
        if !source_integrity_ok {
            return Err(format!(
                "Source integrity check failed: device prefix changed during imaging (pre={pre_source_hash}, post={post_source_hash})"
            ));
        }

        crate::progress::set_imaging_summary(final_summary.clone());
        super::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
    }

    fn run_raw(
        &self,
        source: &str,
        cancel_flag: &std::sync::atomic::AtomicBool,
        started: Instant,
        pre_source_hash: String,
        bad_log: &mut BadSectorLog,
    ) -> Result<(String, ImagingSummary), String> {
        let mut src_file = File::open(source)
            .map_err(|e| format!("Cannot open source {source}: {e}"))?;
        let src_size = crate::block_device::device_size(source)?;
        let has_known_size = src_size > 0;

        let mut total_written: u64 = 0;
        let mut part_num: u32 = 0;
        let mut hasher = sha2::Sha256::new();

        let stem = self.destination.file_stem().unwrap_or_default().to_string_lossy();
        let dir = self.destination.parent().unwrap_or(Path::new("."));

        loop {
            if cancel_flag.load(Ordering::SeqCst) {
                return Err("CANCELLED".into());
            }

            part_num += 1;
            let out_name = if self.split_size.is_some() && part_num > 1 {
                format!("{}.{:05}", stem, part_num)
            } else if self.split_size.is_some() && part_num == 1 {
                format!("{}.00001", stem)
            } else {
                stem.to_string()
            };
            let out_path = dir.join(&out_name);

            let dst = OpenOptions::new().write(true).create(true).truncate(true).open(&out_path)
                .map_err(|e| format!("Cannot create {}: {}", out_path.display(), e))?;
            let mut writer = BufWriter::with_capacity(super::hashing::HASH_BUFFER_SIZE, dst);

            let mut part_written: u64 = 0;
            let split_limit = self.split_size.unwrap_or(u64::MAX);
            let mut buf = vec![0u8; super::hashing::HASH_BUFFER_SIZE];

            loop {
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("CANCELLED".into());
                }
                let byte_offset = src_file
                    .stream_position()
                    .map_err(|e| format!("Stream position: {e}"))?;
                let to_read = buf.len().min((split_limit - part_written) as usize);
                if to_read == 0 {
                    break;
                }
                let n = read_resilient(
                    &mut src_file,
                    &mut buf[..to_read],
                    byte_offset,
                    DEFAULT_SECTOR_SIZE,
                    bad_log,
                )?;
                if n == 0 {
                    break;
                }

                let chunk = &buf[..n];
                writer.write_all(chunk).map_err(|e| format!("Write error: {}", e))?;
                hasher.update(chunk);
                part_written += n as u64;
                total_written += n as u64;

                let status_suffix = if bad_log.error_sectors > 0 {
                    format!(" · {} bad sectors (zeroed)", bad_log.error_sectors)
                } else {
                    String::new()
                };
                let pct = if has_known_size && src_size > 0 {
                    (total_written as f64 / src_size as f64) * 100.0
                } else {
                    0.0
                };
                super::progress::update_progress(
                    pct,
                    &format!(
                        "Imaging: {} / {}{}",
                        crate::block_device::format_capacity(total_written),
                        if has_known_size {
                            crate::block_device::format_capacity(src_size)
                        } else {
                            "unknown".into()
                        },
                        status_suffix
                    ),
                    total_written,
                    if has_known_size { src_size } else { 0 },
                );

                if part_written >= split_limit {
                    break;
                }
            }

            writer.flush().map_err(|e| e.to_string())?;

            if self.split_size.is_none() {
                break;
            }
            if part_written < split_limit {
                break;
            }
            if has_known_size && total_written >= src_size {
                break;
            }
        }

        let hash = format!("{:x}", hasher.finalize());
        let duration = started.elapsed().as_secs_f64();
        let sectors_read = total_written / 512;

        if self.verify {
            super::progress::update_progress(
                99.0,
                "Verifying image hash…",
                total_written,
                if has_known_size { src_size } else { total_written },
            );
            if self.split_size.is_some() {
                verify_split_raw_parts(dir.as_ref(), &stem, part_num, &hash)?;
            } else {
                let verify_hash = crate::imaging_format::hash_file_sha256(&self.destination)?;
                if verify_hash != hash {
                    return Err(format!(
                        "Verify failed: stream hash {hash} != file hash {verify_hash}"
                    ));
                }
            }

            let first_part = if self.split_size.is_some() {
                dir.join(format!("{}.00001", stem))
            } else {
                self.destination.clone()
            };
            let image_prefix_hash = hash_prefix(&first_part, SOURCE_INTEGRITY_BYTES)?;
            if image_prefix_hash != pre_source_hash {
                return Err(format!(
                    "Image prefix verify failed: source {pre_source_hash} != image {image_prefix_hash}"
                ));
            }
        }

        let summary = ImagingSummary {
            sha256: hash.clone(),
            sectors_read,
            avg_speed_bytes_per_sec: if duration > 0.0 {
                total_written as f64 / duration
            } else {
                0.0
            },
            error_sectors: bad_log.error_sectors,
            duration_secs: duration,
            source_integrity_ok: true,
            bytes_written: total_written,
            bad_sectors_log: None,
        };

        Ok((hash, summary))
    }
}

#[cfg(test)]
mod disk_list_tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_whole_disk_filter() {
        assert!(is_macos_whole_disk("/dev/disk2"));
        assert!(!is_macos_whole_disk("/dev/disk2s1"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parse_diskutil_list_skips_partitions() {
        let sample = r#"
/dev/disk0 (internal, physical):
   #:                       TYPE NAME                    SIZE       IDENTIFIER
   0:      GUID_partition_scheme                        *500.3 GB   disk0
/dev/disk2 (external, physical):
   #:                       TYPE NAME                    SIZE       IDENTIFIER
   0:      FDisk_partition_scheme                        *16.1 GB   disk2
"#;
        let disks = parse_diskutil_list_text(sample);
        assert_eq!(disks.len(), 2);
        assert_eq!(disks[0].device, "/dev/disk0");
        assert_eq!(disks[1].device, "/dev/disk2");
    }
}
