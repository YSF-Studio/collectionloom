use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use sha2::Digest;

use crate::imaging_format::ImageFormat;

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
        let output = Command::new("diskutil")
            .args(["list", "external", "physical"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut disks = vec![];
        let mut current: Option<(String, String)> = None;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("/dev/disk") {
                if let Some((device, size_hint)) = current.take() {
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
                let parts: Vec<&str> = trimmed.split(':').collect();
                let device = parts[0].trim().to_string();
                let hint = parts.get(1).unwrap_or(&"").trim().to_string();
                current = Some((device, hint));
            }
        }
        if let Some((device, size_hint)) = current {
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
        Ok(disks)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn list() -> Result<Vec<Self>, String> {
        Ok(vec![])
    }
}

#[cfg(target_os = "macos")]
fn disk_info_macos(device: &str) -> Result<DiskInfo, String> {
    use std::process::Command;
    let output = Command::new("diskutil")
        .args(["info", "-plist", device])
        .output()
        .map_err(|e| e.to_string())?;
    let plist = String::from_utf8_lossy(&output.stdout);
    let size = plist
        .lines()
        .skip_while(|l| !l.contains("TotalSize"))
        .nth(1)
        .and_then(|l| l.trim().trim_start_matches("<integer>").trim_end_matches("</integer>").parse().ok())
        .unwrap_or(0);
    let model = plist
        .lines()
        .skip_while(|l| !l.contains("MediaName"))
        .nth(1)
        .map(|l| {
            l.trim()
                .trim_start_matches("<string>")
                .trim_end_matches("</string>")
                .to_string()
        })
        .unwrap_or_else(|| "Unknown".into());
    Ok(DiskInfo {
        device: device.to_string(),
        model,
        size_bytes: size,
        sector_size: 512,
        is_ssd: true,
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
        match self.format {
            ImageFormat::E01 => {
                crate::imaging_format::acquire_e01(&source, &self.destination, cancel_flag)
            }
            ImageFormat::Aff4 => crate::imaging_format::acquire_aff4(
                &source,
                &self.destination,
                self.split_size,
                self.verify,
                cancel_flag,
            ),
            ImageFormat::Raw => self.run_raw(&source, cancel_flag),
        }
    }

    fn run_raw(&self, source: &str, cancel_flag: &std::sync::atomic::AtomicBool) -> Result<String, String> {
        let src = File::open(source)
            .map_err(|e| format!("Cannot open source {source}: {e}"))?;
        let src_size = crate::block_device::device_size(source)?;
        let mut reader = BufReader::with_capacity(super::hashing::HASH_BUFFER_SIZE, src);
        let has_known_size = src_size > 0;

        let mut total_written: u64 = 0;
        let mut part_num: u32 = 0;
        let mut hasher = sha2::Sha256::new();

        // Determine output path
        let stem = self.destination.file_stem().unwrap_or_default().to_string_lossy();
        let dir = self.destination.parent().unwrap_or(Path::new("."));

        loop {
            if cancel_flag.load(Ordering::SeqCst) {
                return Err("CANCELLED".into());
            }

            // Open output part
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
                let n = reader.read(&mut buf).map_err(|e| format!("Read error: {}", e))?;
                if n == 0 { break; }

                let chunk = &buf[..n];
                writer.write_all(chunk).map_err(|e| format!("Write error: {}", e))?;
                hasher.update(chunk);
                part_written += n as u64;
                total_written += n as u64;

                let pct = if has_known_size && src_size > 0 {
                    (total_written as f64 / src_size as f64) * 100.0
                } else {
                    0.0
                };
                super::progress::update_progress(
                    pct,
                    &format!(
                        "Imaging: {} / {}",
                        crate::block_device::format_capacity(total_written),
                        if has_known_size {
                            crate::block_device::format_capacity(src_size)
                        } else {
                            "unknown".into()
                        }
                    ),
                    total_written,
                    if has_known_size { src_size } else { 0 },
                );

                if part_written >= split_limit { break; }
            }

            writer.flush().map_err(|e| e.to_string())?;

            if self.split_size.is_none() {
                break;
            }
            // EOF: last read returned 0 before filling this part
            if part_written < split_limit {
                break;
            }
            if has_known_size && total_written >= src_size {
                break;
            }
        }

        let hash = format!("{:x}", hasher.finalize());

        if self.verify && self.split_size.is_none() {
            super::progress::update_progress(
                99.0,
                "Verifying image hash…",
                total_written,
                if has_known_size { src_size } else { total_written },
            );
            let verify_hash = crate::imaging_format::hash_file_sha256(&self.destination)?;
            if verify_hash != hash {
                return Err(format!(
                    "Verify failed: stream hash {hash} != file hash {verify_hash}"
                ));
            }
        } else if self.verify && self.split_size.is_some() {
            super::progress::update_progress(
                99.0,
                "Split image — stream SHA-256 recorded (multi-part verify via manifest)",
                total_written,
                if has_known_size { src_size } else { total_written },
            );
        }

        super::progress::finish_progress(Ok(hash.clone()));
        Ok(hash)
    }
}
