//! YSF Core — Shared forensic library for CollectionLoom
//!
//! All modules are pure Rust with no Tauri dependencies — testable independently.

pub mod progress;
pub mod hashing;
pub mod crypto;
pub mod evidence;
pub mod encryption_detect;
pub mod block_device;
pub mod imaging;
pub mod imaging_format;
pub mod ewf;
pub mod aff4_native;
pub mod write_blocker;
pub mod ram;
pub mod mobile;
pub mod cloud;
mod aws_sigv4;
pub mod network;
pub mod archive;
pub mod ntfs;
pub mod carving;
pub mod report;
pub mod snapshot;
pub mod preview;
pub mod hpa_dco;
pub mod bad_sector;
pub mod timestamp;
pub mod storage_check;
pub mod evidence_hash;
pub mod preflight;
pub mod portable;

// Re-export commonly used types
pub use progress::{ProgressState, CancelFlag, set_cancel_flag, is_cancelled, ImagingSummary};
pub use hashing::{multi_hash, compute_entropy, check_magic_bytes, HASH_BUFFER_SIZE};
pub use crypto::{sign_data, verify_signature, generate_keypair, KeypairStore};
pub use evidence::{EvidenceId, ActionLog, ChainOfCustody, generate_qr_label};
pub use timestamp::{TimestampToken, create_local_timestamp, create_timestamp_with_optional_tsa, verify_local_timestamp};
pub use storage_check::{StorageCheckReport, verify_acquisition_storage};
pub use evidence_hash::{EvidenceHashReport, hash_and_verify_evidence};
pub use preflight::{PreflightCategory, PreflightCheck, PreflightReport, run_preflight};
pub use portable::{
    PortableStatus, ResolvedTool, command as portable_command, portable_status, resolve_kit_root,
    resolve_tool, same_volume, tool_available, tool_path, tools_dir,
};
pub use hpa_dco::{HpaDcoReport, detect as detect_hpa_dco};
pub use bad_sector::{BadSectorLog, read_resilient, DEFAULT_SECTOR_SIZE};
pub use encryption_detect::{EncryptionReport, FdeType, scan_encryption};
pub use imaging::{DiskImager, AcquisitionState, DiskInfo};
pub use imaging_format::ImageFormat;
pub use write_blocker::{
    enable_write_blocker, disable_write_blocker, check_write_blocker, check_write_blocker_status,
    WriteBlockerStatus,
};
pub use archive::{
    forensic_load, generate_forensic_report, ForensicReport, FileEntry, Anomaly, Threat,
    FORMATS_SUPPORTED,
};
pub use ntfs::{parse_mft, MftEntry, FileAttribute, DeletedFile};
pub use carving::{carve_files, CarvingResult, CarvedFile, MAGIC_SIGNATURES};
pub use report::generate_pdf_report;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

/// Global cancel flag shared across all modules
pub static CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
pub static PROGRESS_STATE: Lazy<Mutex<ProgressState>> = Lazy::new(|| {
    Mutex::new(ProgressState::default())
});
pub static OPERATION_RESULT: Lazy<Mutex<Option<Result<String, String>>>> = Lazy::new(|| Mutex::new(None));

/// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SUITE_NAME: &str = "YSF Forensic Suite";
