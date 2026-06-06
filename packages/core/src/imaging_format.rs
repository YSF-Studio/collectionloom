use std::path::Path;
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use crate::aff4_native::{aff4_path, Aff4Writer};
use crate::ewf::{e01_path, EwfWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Raw,
    E01,
    Aff4,
}

impl ImageFormat {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "e01" | "ewf" => Self::E01,
            "aff4" => Self::Aff4,
            _ => Self::Raw,
        }
    }
}

/// Native Rust E01 writer (no ewfacquire).
pub fn acquire_e01(
    source: &str,
    destination: &Path,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    let path = e01_path(destination);
    EwfWriter::new(&path).acquire(source, cancel_flag)
}

/// Native Rust AFF4-L ZIP container (no aff4acquire).
pub fn acquire_aff4(
    source: &str,
    destination: &Path,
    split_size: Option<u64>,
    verify: bool,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    let _ = split_size;
    let path = aff4_path(destination);
    let hash = Aff4Writer::new(&path).acquire(source, cancel_flag)?;
    if verify {
        let verify_hash = hash_file_sha256(&path)?;
        if verify_hash != hash {
            return Err(format!("AFF4 verify failed: {hash} != {verify_hash}"));
        }
    }
    Ok(hash)
}

pub fn hash_file_sha256(path: &Path) -> Result<String, String> {
    use sha2::Digest;
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = sha2::Sha256::new();
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

/// Prefer raw block device on macOS (`/dev/rdiskN`).
pub fn normalize_block_source(source: &str) -> String {
    if cfg!(target_os = "macos") && source.starts_with("/dev/disk") && !source.starts_with("/dev/rdisk")
    {
        source.replacen("/dev/disk", "/dev/rdisk", 1)
    } else {
        source.to_string()
    }
}
