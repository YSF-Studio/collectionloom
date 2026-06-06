use serde::Serialize;
use ts_rs::TS;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/ImagingSummary.ts")]
pub struct ImagingSummary {
    pub sha256: String,
    pub sectors_read: u64,
    pub avg_speed_bytes_per_sec: f64,
    pub error_sectors: u64,
    pub duration_secs: f64,
    pub source_integrity_ok: bool,
    pub bytes_written: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bad_sectors_log: Option<String>,
}

#[derive(Clone, Debug, Serialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../collectionloom/src/lib/generated/ProgressState.ts")]
pub struct ProgressState {
    pub percent: f64,
    pub status: String,
    pub is_done: bool,
    pub error: Option<String>,
    pub eta_secs: Option<f64>,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ImagingSummary>,
}

pub struct CancelFlag {
    flag: Arc<AtomicBool>,
}

impl CancelFlag {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }
    pub fn cancel(&self) { self.flag.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.flag.load(Ordering::SeqCst) }
    pub fn reset(&self) { self.flag.store(false, Ordering::SeqCst); }
    pub fn clone_arc(&self) -> Arc<AtomicBool> { self.flag.clone() }
}

pub fn set_cancel_flag(flag: Arc<AtomicBool>) {
    *CANCEL_FLAG_MUTEX.lock().unwrap() = Some(flag);
}

pub fn is_cancelled() -> bool {
    CANCEL_FLAG_MUTEX.lock().unwrap()
        .as_ref().map(|f| f.load(Ordering::SeqCst)).unwrap_or(false)
}

/// Update progress state (thread-safe)
pub fn update_progress(percent: f64, status: &str, bytes: u64, total: u64) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        p.percent = percent;
        p.status = status.to_string();
        p.bytes_processed = bytes;
        p.total_bytes = total;
    }
}

/// Attach imaging summary to progress (shown after acquisition).
pub fn set_imaging_summary(summary: ImagingSummary) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        p.summary = Some(summary);
    }
}

/// Mark operation as done
pub fn finish_progress(result: Result<String, String>) {
    if let Ok(mut p) = super::PROGRESS_STATE.lock() {
        p.is_done = true;
        p.percent = 100.0;
        p.status = "Complete".to_string();
        match &result {
            Ok(_) => p.error = None,
            Err(e) => p.error = Some(e.clone()),
        }
    }
    *super::OPERATION_RESULT.lock().unwrap() = Some(result);
}

use once_cell::sync::Lazy;
static CANCEL_FLAG_MUTEX: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));
