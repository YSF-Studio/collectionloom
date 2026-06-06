use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;

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

fn tool_in_path(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// E01 via libewf `ewfacquire` when installed (brew install libewf).
pub fn acquire_e01(
    source: &str,
    destination: &Path,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    if !tool_in_path("ewfacquire") {
        return Err(
            "E01 imaging requires ewfacquire (libewf-tools). Install: brew install libewf".into(),
        );
    }

    let src_size = std::fs::metadata(source)
        .map_err(|e| format!("Cannot stat source {source}: {e}"))?
        .len();

    let dest_str = destination.to_string_lossy();
    let stem = destination
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "evidence".into());

    // ewfacquire -t source -o basename -C Best -c best -d sha256
    let status = Command::new("ewfacquire")
        .args([
            "-t",
            source,
            "-o",
            &stem,
            "-C",
            "Best",
            "-c",
            "best",
            "-d",
            "sha256",
            "-D",
            "CollectionLoom acquisition",
        ])
        .current_dir(
            destination
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        )
        .status()
        .map_err(|e| format!("Failed to run ewfacquire: {e}"))?;

    if cancel_flag.load(Ordering::SeqCst) {
        return Err("CANCELLED".into());
    }
    if !status.success() {
        return Err(format!(
            "ewfacquire failed (exit {:?}). Ensure source is a block device and destination is writable.",
            status.code()
        ));
    }

    let e01_path = if dest_str.ends_with(".E01") || dest_str.ends_with(".e01") {
        destination.to_path_buf()
    } else {
        destination.with_extension("E01")
    };

    if !e01_path.exists() {
        // Some builds append .E01 to stem automatically
        let alt = destination
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{stem}.E01"));
        if alt.exists() {
            std::fs::rename(&alt, &e01_path).map_err(|e| e.to_string())?;
        } else {
            return Err(format!(
                "E01 output not found at {} — check ewfacquire logs",
                e01_path.display()
            ));
        }
    }

    super::progress::update_progress(100.0, "E01 acquisition complete", src_size, src_size);
    hash_file_sha256(&e01_path)
}

/// AFF4 via `aff4acquire` when installed, otherwise RAW + sidecar manifest.
pub fn acquire_aff4(
    source: &str,
    destination: &Path,
    split_size: Option<u64>,
    verify: bool,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    if tool_in_path("aff4acquire") {
        return acquire_aff4_native(source, destination, cancel_flag);
    }

    // Sidecar fallback: RAW stream + AFF4-style YAML manifest for handover tooling
    let raw_dest = if destination.extension().map(|e| e == "aff4").unwrap_or(false) {
        destination.with_extension("dd")
    } else {
        destination.to_path_buf()
    };

    let mut imager = super::DiskImager::new(source, &raw_dest);
    imager.split_size = split_size;
    imager.verify = verify;
    let hash = imager.run(cancel_flag)?;

    let sidecar = raw_dest.with_extension("aff4.yaml");
    let manifest = format!(
        "# CollectionLoom AFF4 sidecar (native AFF4 pending aff4acquire)\n\
         format: aff4-sidecar-v1\n\
         source: {source}\n\
         raw_image: {}\n\
         sha256: {hash}\n\
         acquired_at: {}\n\
         note: Install aff4acquire for native AFF4 containers\n",
        raw_dest.display(),
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    );
    std::fs::write(&sidecar, manifest).map_err(|e| e.to_string())?;
    Ok(hash)
}

fn acquire_aff4_native(
    source: &str,
    destination: &Path,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    let src_size = std::fs::metadata(source)
        .map_err(|e| format!("Cannot stat source {source}: {e}"))?
        .len();

    let dest_str = destination.to_string_lossy().to_string();
    let mut child = Command::new("aff4acquire")
        .args(["-i", source, "-o", &dest_str])
        .spawn()
        .map_err(|e| format!("Failed to run aff4acquire: {e}"))?;

    loop {
        if cancel_flag.load(Ordering::SeqCst) {
            let _ = child.kill();
            return Err("CANCELLED".into());
        }
        if let Ok(meta) = std::fs::metadata(destination) {
            let written = meta.len();
            let pct = if src_size > 0 {
                (written as f64 / src_size as f64) * 100.0
            } else {
                0.0
            };
            super::progress::update_progress(
                pct.min(99.0),
                &format!(
                    "AFF4 imaging: {:.1} GB / {:.1} GB",
                    written as f64 / 1e9,
                    src_size as f64 / 1e9
                ),
                written,
                src_size,
            );
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(format!(
                        "aff4acquire failed (exit {:?})",
                        status.code()
                    ));
                }
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(500)),
            Err(e) => return Err(format!("aff4acquire wait failed: {e}")),
        }
    }

    hash_file_sha256(destination)
}

pub fn hash_file_sha256(path: &Path) -> Result<String, String> {
    use sha2::Digest;
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; super::hashing::HASH_BUFFER_SIZE];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
