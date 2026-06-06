//! Post-acquisition hashing and integrity verification (ISO 27037 §6.5).

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceHashReport {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub verified: bool,
    pub verify_passes: u32,
}

fn sha256_file(path: &Path) -> Result<(String, u64), String> {
    let mut file = File::open(path).map_err(|e| format!("Open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; crate::hashing::HASH_BUFFER_SIZE];
    let mut size: u64 = 0;
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("Read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

/// Hash file twice; `verified` is true when both passes match (detects read instability).
pub fn hash_and_verify_evidence(path: &str) -> Result<EvidenceHashReport, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("Evidence file not found: {path}"));
    }
    let (hash1, size1) = sha256_file(p)?;
    let (hash2, size2) = sha256_file(p)?;
    let verified = hash1 == hash2 && size1 == size2;
    Ok(EvidenceHashReport {
        path: path.to_string(),
        sha256: hash1,
        size_bytes: size1,
        verified,
        verify_passes: if verified { 2 } else { 1 },
    })
}
