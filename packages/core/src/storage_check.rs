//! Pre-acquisition checks for evidence output paths (RAM/network/mobile).

use serde::Serialize;
use ts_rs::TS;
use std::path::Path;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/StorageCheckReport.ts")]
pub struct StorageCheckReport {
    pub ok: bool,
    pub output_path: String,
    pub notes: String,
    pub same_volume_as_source: bool,
}

/// Verify output path is suitable for volatile evidence (not the source device path).
pub fn verify_acquisition_storage(output: &str, source_device: Option<&str>) -> StorageCheckReport {
    let mut issues = Vec::new();
    let path = Path::new(output);
    let parent = path.parent().unwrap_or(Path::new("."));

    if output.trim().is_empty() {
        issues.push("Output path is empty");
    }

    if let Some(src) = source_device.filter(|s| !s.is_empty()) {
        let src_norm = src.trim();
        let out_norm = output.trim();
        if out_norm.starts_with(src_norm) || src_norm.starts_with(out_norm) {
            issues.push("Output must not be on the source evidence device");
        }
        if crate::portable::same_volume(output, Some(src)) {
            issues.push("Output is on the same volume as the source device — use external evidence storage");
        }
        if cfg!(unix) {
            if out_norm.starts_with("/dev/") {
                issues.push("Output must not be a block device path");
            }
        }
    }

    if !parent.exists() {
        if std::fs::create_dir_all(parent).is_err() {
            issues.push("Cannot create output directory");
        }
    } else if !parent.is_dir() {
        issues.push("Output parent is not a directory");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if parent.exists() {
            if let Ok(meta) = std::fs::metadata(parent) {
                if meta.mode() & 0o200 == 0 {
                    issues.push("Output directory is not writable");
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        let probe = parent.join(".cl_write_probe");
        if std::fs::write(&probe, b"1").is_ok() {
            let _ = std::fs::remove_file(probe);
        } else {
            issues.push("Output directory is not writable");
        }
    }

    let same_vol = source_device
        .filter(|s| !s.is_empty())
        .is_some_and(|src| crate::portable::same_volume(output, Some(src)));

    let ok = issues.is_empty();
    StorageCheckReport {
        ok,
        output_path: output.to_string(),
        notes: if ok {
            "Output storage OK for acquisition".into()
        } else {
            issues.join("; ")
        },
        same_volume_as_source: same_vol,
    }
}
