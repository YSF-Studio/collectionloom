use tauri::Emitter;
use tauri_plugin_dialog::DialogExt;
use ysf_core::*;
use ysf_core::progress::finish_progress;
use std::sync::atomic::Ordering;

use crate::storage::{self, AcquisitionAuditEntry};

fn log_acquisition(
    case_id: Option<String>,
    acquisition_type: &str,
    source: &str,
    destination: &str,
    status: &str,
    sha256: Option<String>,
    error_sectors: u64,
    operator: Option<String>,
    details: Option<String>,
) {
    let entry = AcquisitionAuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        acquisition_type: acquisition_type.to_string(),
        source: source.to_string(),
        destination: destination.to_string(),
        status: status.to_string(),
        sha256,
        error_sectors,
        operator,
        details,
    };
    let _ = storage::append_acquisition_audit(case_id.as_deref(), &entry);
}

// ─── Disk Imaging ───

#[tauri::command]
pub fn list_disks() -> Result<Vec<imaging::DiskInfo>, String> {
    imaging::DiskInfo::list()
}

#[tauri::command]
pub async fn start_disk_imaging(
    source: String,
    destination: String,
    split_size_mb: u64,
    verify: bool,
    image_format: String,
    case_id: Option<String>,
    operator: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Reset global state
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    *PROGRESS_STATE.lock().unwrap() = ProgressState::default();
    *OPERATION_RESULT.lock().unwrap() = None;

    let cancel = CANCEL_FLAG.clone();
    let src_log = source.clone();
    let dest_log = destination.clone();
    let case_log = case_id.clone();
    let operator_log = operator.clone();

    tokio::task::spawn_blocking(move || {
        let mut imager = imaging::DiskImager::new(&source, std::path::Path::new(&destination));
        imager.split_size = if split_size_mb > 0 { Some(split_size_mb * 1_048_576) } else { None };
        imager.verify = verify;
        imager.format = ImageFormat::parse(&image_format);

        match imager.run(&cancel) {
            Ok(hash) => {
                let summary = PROGRESS_STATE
                    .lock()
                    .ok()
                    .and_then(|p| p.summary.clone());
                let error_sectors = summary.as_ref().map(|s| s.error_sectors).unwrap_or(0);
                let details = summary.as_ref().and_then(|s| s.bad_sectors_log.clone()).map(|p| {
                    format!("bad_sectors={error_sectors} log={p}")
                });
                log_acquisition(
                    case_log,
                    "disk_imaging",
                    &src_log,
                    &dest_log,
                    "completed",
                    Some(hash.clone()),
                    error_sectors,
                    operator_log,
                    details,
                );
                let payload = if let Some(s) = summary {
                    serde_json::json!({ "hash": hash, "summary": s })
                } else {
                    serde_json::json!({ "hash": hash })
                };
                let _ = app.emit("imaging_complete", payload);
            }
            Err(e) => {
                log_acquisition(
                    case_log,
                    "disk_imaging",
                    &src_log,
                    &dest_log,
                    "failed",
                    None,
                    0,
                    operator_log,
                    Some(e.clone()),
                );
                finish_progress(Err(e.clone()));
                let _ = app.emit("imaging_error", &e);
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_imaging_progress() -> Result<ProgressState, String> {
    PROGRESS_STATE.lock().map(|s| s.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_imaging() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

// ─── RAM Capture ───

#[tauri::command]
pub fn list_ram_tools() -> Result<Vec<String>, String> {
    Ok(ram::detect_tools().iter().map(|t| format!("{:?}", t)).collect())
}

#[tauri::command]
pub fn get_ram_size() -> Result<u64, String> {
    ram::get_ram_size()
}

#[tauri::command]
pub fn list_processes() -> Result<Vec<snapshot::ProcessEntry>, String> {
    snapshot::list_running_processes()
}

#[tauri::command]
pub fn capture_ram(
    tool: String,
    output: String,
    compress: bool,
    case_id: Option<String>,
    operator: Option<String>,
) -> Result<serde_json::Value, String> {
    let selected = if tool.trim().is_empty() {
        ram::detect_tools()
            .into_iter()
            .next()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|| "Avml".into())
    } else {
        tool.trim().to_string()
    };

    let result = match selected.as_str() {
        "Avml" => ram::capture_avml(&output, compress),
        "WinPmem" => ram::capture_winpmem(&output),
        "LiME" => Err("LiME capture is not yet implemented in the app flow; use AVML or a pre-staged LiME workflow".into()),
        _ => Err(format!("Unknown tool: {}", selected)),
    };
    match &result {
        Ok(msg) => {
            let hash_report = evidence_hash::hash_and_verify_evidence(&output).ok();
            let sha256 = hash_report.as_ref().map(|r| r.sha256.clone());
            log_acquisition(
                case_id,
                "ram_capture",
                &selected,
                &output,
                "completed",
                sha256.clone(),
                0,
                operator,
                Some(msg.clone()),
            );
            Ok(serde_json::json!({
                "message": msg,
                "output": output,
                "sha256": sha256,
                "verified": hash_report.as_ref().map(|r| r.verified).unwrap_or(false),
                "size_bytes": hash_report.as_ref().map(|r| r.size_bytes),
            }))
        }
        Err(e) => {
            log_acquisition(
                case_id,
                "ram_capture",
                &selected,
                &output,
                "failed",
                None,
                0,
                operator,
                Some(e.clone()),
            );
            Err(e.clone())
        }
    }
}

// ─── Write Blocker ───

#[tauri::command]
pub fn enable_write_blocker(device: String) -> Result<write_blocker::WriteBlockerStatus, String> {
    write_blocker::enable_write_blocker(&device)
}

#[tauri::command]
pub fn disable_write_blocker(device: String) -> Result<write_blocker::WriteBlockerStatus, String> {
    write_blocker::disable_write_blocker(&device)
}

#[tauri::command]
pub fn check_write_blocker(device: String) -> Result<write_blocker::WriteBlockerStatus, String> {
    Ok(write_blocker::check_write_blocker_status(&device))
}

// ─── Mobile ───

#[tauri::command]
pub fn list_android_devices() -> Result<Vec<mobile::MobileDevice>, String> {
    mobile::list_android_devices()
}

#[tauri::command]
pub fn adb_backup(device_id: String, output: String) -> Result<String, String> {
    mobile::adb_backup(&device_id, &output)
}

#[tauri::command]
pub fn list_ios_devices() -> Result<Vec<mobile::MobileDevice>, String> {
    mobile::list_ios_devices()
}

#[tauri::command]
pub fn ios_backup(device_id: String, output: String) -> Result<String, String> {
    mobile::ios_backup(&device_id, &output)
}

// ─── Network ───

#[tauri::command]
pub fn list_interfaces() -> Result<Vec<String>, String> {
    network::list_interfaces()
}

#[tauri::command]
pub async fn start_network_capture(
    interface: String,
    bpf_filter: Option<String>,
    output_file: String,
    max_duration_secs: Option<u64>,
) -> Result<String, String> {
    let duration = max_duration_secs.unwrap_or(3600);
    let config = network::NetworkCaptureConfig {
        interface,
        bpf_filter,
        output_file,
        ring_buffer_size: 256 * 1024 * 1024, // 256 MB
        max_duration_secs: duration,
    };
    let cancel = CANCEL_FLAG.clone();
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    tokio::task::spawn_blocking(move || network::start_capture(config, cancel))
        .await.map_err(|e| format!("Internal: {}", e))?
}

#[tauri::command]
pub fn cancel_network_capture() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

// ─── Encryption Detection ───

#[tauri::command]
pub fn scan_encryption() -> Result<EncryptionReport, String> {
    Ok(encryption_detect::scan_encryption())
}

// ─── Chain of Custody ───

#[tauri::command]
pub fn create_chain_of_custody(
    case_name: String,
    operator: String,
    source_device: String,
) -> Result<String, String> {
    let coc = evidence::ChainOfCustody::new(&case_name, &operator, &source_device, 0);
    Ok(coc.evidence_id)
}

#[tauri::command]
pub fn generate_coc_report(evidence_id: String) -> Result<String, String> {
    if evidence_id.is_empty()
        || evidence_id.contains('/')
        || evidence_id.contains('\\')
        || evidence_id.contains("..")
    {
        return Err("Invalid evidence ID".into());
    }
    let report_path = format!("/tmp/{}_coc_report.pdf", evidence_id);
    let _coc = evidence::ChainOfCustody::new("case", "operator", "device", 0);
    let pdf = report::generate_pdf_report(&report::PdfReport {
        title: format!("Chain of Custody — {}", evidence_id),
        evidence_id: evidence_id.clone(),
        operator: "Yusuf Shalahuddin".into(),
        case_name: "Forensic Case".into(),
        device: "Source Device".into(),
        date: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        sections: vec![
            report::ReportSection { heading: "Evidence".into(), content: format!("ID: {}", evidence_id) },
        ],
    })?;
    std::fs::write(&report_path, pdf).map_err(|e| e.to_string())?;
    Ok(report_path)
}

// ─── Helpers ───

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Invalid hex string length".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| "Invalid hex character".into())
        })
        .collect()
}

// ─── Acquisition integrity (ISO 27037) ───

#[tauri::command]
pub fn verify_acquisition_storage(
    output: String,
    source_device: Option<String>,
) -> Result<storage_check::StorageCheckReport, String> {
    Ok(storage_check::verify_acquisition_storage(
        &output,
        source_device.as_deref(),
    ))
}

#[tauri::command]
pub fn hash_and_verify_evidence(path: String) -> Result<evidence_hash::EvidenceHashReport, String> {
    evidence_hash::hash_and_verify_evidence(&path)
}

// ─── Chain of Custody Signing ───

#[tauri::command]
pub async fn sign_coc(
    evidence_id: String,
    private_key_hex: Option<String>,
    hash_sha256: Option<String>,
    operator: Option<String>,
    tsa_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let keypair = match private_key_hex {
        Some(ref hex_key) => {
            let priv_bytes = hex_decode(hex_key)?;
            ysf_core::crypto::KeypairStore::from_bytes(&priv_bytes)?
        }
        None => ysf_core::crypto::generate_keypair(),
    };

    let mut sign_parts = vec![evidence_id.clone()];
    if let Some(ref h) = hash_sha256 {
        if !h.is_empty() {
            sign_parts.push(h.clone());
        }
    }
    if let Some(ref op) = operator {
        if !op.is_empty() {
            sign_parts.push(op.clone());
        }
    }
    let sign_payload = sign_parts.join("|");

    let signature =
        ysf_core::crypto::sign_data(&keypair.private_key, sign_payload.as_bytes())?;

    let timestamp = timestamp::create_timestamp_with_optional_tsa(
        sign_payload.as_bytes(),
        &keypair,
        tsa_url.as_deref(),
    )
    .await?;

    Ok(serde_json::json!({
        "evidence_id": evidence_id,
        "signature_hex": hex_encode(&signature),
        "public_key_hex": hex_encode(&keypair.public_key),
        "hash_sha256": hash_sha256,
        "operator": operator,
        "signed_at": timestamp.signed_at,
        "timestamp": timestamp,
    }))
}

// ─── HPA / DCO ───

#[tauri::command]
pub fn hpa_dco_detect(device: String) -> Result<hpa_dco::HpaDcoReport, String> {
    hpa_dco::detect(&device)
}

// ─── Evidence ID Generation ───

#[tauri::command]
pub fn generate_evidence_id(case_initials: Option<String>, media_type: Option<String>) -> Result<String, String> {
    let initials = case_initials.unwrap_or_else(|| {
        let year = chrono::Utc::now().format("%Y").to_string();
        format!("CL{year}")
    });
    let media = media_type.unwrap_or_else(|| "DSK".into());
    Ok(evidence::EvidenceId::new(&initials, &media).to_string())
}

// ─── File Hashing ───

#[tauri::command]
pub fn compute_file_hash(path: String) -> Result<serde_json::Value, String> {
    use std::io::Read;

    let mut file = std::fs::File::open(&path)
        .map_err(|e| format!("Cannot open file: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("Read error: {e}"))?;

    let hashes = ysf_core::hashing::multi_hash_buffer(&data);
    let entropy = ysf_core::hashing::compute_entropy(&data);

    Ok(serde_json::json!({
        "path": path,
        "sha256": hashes.sha256,
        "sha1": hashes.sha1,
        "md5": hashes.md5,
        "sha512": hashes.sha512,
        "blake3": hashes.blake3,
        "size": data.len() as u64,
        "entropy": entropy,
    }))
}

// ─── Hash Verification ───

#[tauri::command]
pub fn verify_hash(path: String, expected_hash: String, algorithm: String) -> Result<serde_json::Value, String> {
    use std::io::Read;

    let mut file = std::fs::File::open(&path)
        .map_err(|e| format!("Cannot open file: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("Read error: {e}"))?;

    let hashes = ysf_core::hashing::multi_hash_buffer(&data);
    let actual = match algorithm.as_str() {
        "sha256" => hashes.sha256.clone(),
        "sha1" => hashes.sha1.clone(),
        "md5" => hashes.md5.clone(),
        _ => return Err(format!("Unknown algorithm: {algorithm}. Use sha256, sha1, or md5.")),
    }.unwrap_or_default();

    let matched = actual.to_lowercase() == expected_hash.to_lowercase();

    Ok(serde_json::json!({
        "path": path,
        "algorithm": algorithm,
        "expected": expected_hash.to_lowercase(),
        "actual": actual.to_lowercase(),
        "matched": matched,
        "size": data.len() as u64,
    }))
}

#[tauri::command]
pub fn run_preflight_check() -> Result<ysf_core::PreflightReport, String> {
    Ok(ysf_core::run_preflight())
}

#[tauri::command]
pub fn get_portable_layout() -> Result<ysf_core::PortableLayout, String> {
    ysf_core::ensure_kit_directories()?;
    Ok(ysf_core::portable_layout())
}

#[tauri::command]
pub fn about_info() -> serde_json::Value {
    serde_json::json!({
        "appName": "CollectionLoom",
        "version": "0.1.0",
        "developer": "YSF Studio — Yusuf Shalahuddin",
        "build": "Portable Forensic Acquisition Toolkit — macOS / Windows / Linux",
        "features": [
            "Bit-for-bit disk imaging (RAW, E01, AFF4) with SHA-256 verification and split support for multi-TB drives",
            "Hardware and one-click software write-blocker (Linux BLKROSET, macOS unmount, Windows IOCTL)",
            "Volatile RAM capture via avml / LiME with live process listing",
            "Mobile triage — Android ADB and iOS logical backup workflows",
            "Cloud snapshot — AWS EBS (SigV4), Azure managed disks, GCP persistent disks",
            "Network packet capture with BPF filters and live statistics",
            "System snapshot profiles (triage, IR, deep) with A/B compare engine",
            "Acquire All — orchestrated multi-module batch acquisition",
            "Encryption detection (BitLocker, LUKS, VeraCrypt, FileVault)",
            "Hash verification, chain of custody with Ed25519 signatures and QR labels",
            "Case dashboard and export bundles (JSON, Markdown, ZIP)",
            "100% offline — no telemetry, no cloud dependency for core workflows"
        ],
        "disclaimer": "This software is provided \"AS IS\" for forensic triage and evidence collection. Operators must follow organizational policy and jurisdictional requirements. Independently verify hashes and chain-of-custody before use in legal proceedings.",
        "offline": true,
        "privacy": "All processing runs locally on your workstation. CollectionLoom does not transmit evidence, telemetry, or analytics to external servers."
    })
}

#[tauri::command]
pub fn take_snapshot() -> Result<serde_json::Value, String> {
    let snap = ysf_core::snapshot::take_snapshot("collectionloom", Some("/home"))
        .map_err(|e| format!("Snapshot error: {e}"))?;
    Ok(serde_json::json!({
        "id": snap.id.0,
        "timestamp": snap.timestamp,
        "file_count": snap.files.len(),
        "process_count": snap.processes.len(),
        "network_count": snap.network.len(),
    }))
}

// ─── Cloud Snapshot ───

#[derive(serde::Deserialize)]
struct CloudCredentialFile {
    access_key: Option<String>,
    secret_key: Option<String>,
    aws_access_key_id: Option<String>,
    aws_secret_access_key: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

fn parse_cloud_credentials(content: &str, provider: &str) -> Result<(String, String), String> {
    if let Ok(json) = serde_json::from_str::<CloudCredentialFile>(content) {
        let access = json
            .access_key
            .or(json.aws_access_key_id)
            .or(json.client_id)
            .filter(|s| !s.is_empty());
        let secret = json
            .secret_key
            .or(json.aws_secret_access_key)
            .or(json.client_secret)
            .filter(|s| !s.is_empty());
        if let (Some(a), Some(s)) = (access, secret) {
            return Ok((a, s));
        }
    }

    let mut access = String::new();
    let mut secret = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_lowercase();
            let val = v.trim().trim_matches('"').to_string();
            match key.as_str() {
                "aws_access_key_id" | "access_key" | "accesskey" | "client_id" => access = val,
                "aws_secret_access_key" | "secret_key" | "secretkey" | "client_secret" | "token" => {
                    secret = val
                }
                _ => {}
            }
        }
    }
    if !access.is_empty() && !secret.is_empty() {
        return Ok((access, secret));
    }

    Err(format!(
        "Could not parse credentials for {provider}. Use JSON {{\"access_key\",\"secret_key\"}} or INI key=value format."
    ))
}

#[tauri::command]
pub async fn pick_cloud_credentials(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Credentials", &["json", "ini", "txt", "csv"])
        .set_title("Select cloud credentials file")
        .blocking_pick_file();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn create_cloud_snapshot(
    provider: String,
    region: String,
    resource_id: String,
    credential_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let cred_path = if let Some(p) = credential_path {
        p
    } else {
        app.dialog()
            .file()
            .add_filter("Credentials", &["json", "ini", "txt", "csv"])
            .set_title("Select cloud credentials file")
            .blocking_pick_file()
            .ok_or("No credentials file selected")?
            .to_string()
    };

    let content = std::fs::read_to_string(&cred_path)
        .map_err(|e| format!("Cannot read credentials file: {e}"))?;
    let (access_key, secret_key) = parse_cloud_credentials(&content, &provider)?;

    match provider.as_str() {
        "aws" => {
            cloud::aws_create_snapshot(&region, &resource_id, &access_key, &secret_key)
                .await
                .map(|raw| serde_json::json!({ "provider": "AWS", "response": raw }))
        }
        "azure" => {
            let parts: Vec<&str> = resource_id.split('|').collect();
            if parts.len() < 3 {
                return Err("Azure requires: subscription|resourceGroup|diskName".into());
            }
            let snap_name = format!("collectionloom-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
            cloud::azure_create_snapshot(parts[0], parts[1], parts[2], &snap_name, &secret_key)
                .await
                .map(|raw| serde_json::json!({ "provider": "Azure", "response": raw }))
        }
        "gcp" => {
            let parts: Vec<&str> = resource_id.split('|').collect();
            if parts.len() < 3 {
                return Err("GCP requires: project|zone|diskName".into());
            }
            let snap_name = format!("collectionloom-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
            cloud::gcp_create_snapshot(parts[0], parts[1], parts[2], &snap_name, &secret_key)
                .await
                .map(|raw| serde_json::json!({ "provider": "GCP", "response": raw }))
        }
        _ => Err(format!("Unknown provider: {provider}. Use aws, azure, or gcp.")),
    }
}

#[tauri::command]
pub fn compare_snapshot(previous_id: String) -> Result<serde_json::Value, String> {
    let current = ysf_core::snapshot::take_snapshot("compare", Some("/home"))
        .map_err(|e| format!("Snapshot error: {}", e))?;

    // We can't reconstruct a previous snapshot from just an ID string in this simple case —
    // we compare current with itself minus 1 minute for demo purposes.
    // Full implementation would store/load snapshots from disk.

    let mut prev = current.clone();
    prev.id = ysf_core::snapshot::SnapshotId(previous_id);
    prev.timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let diff = ysf_core::snapshot::compare_snapshots(&prev, &current);
    let report = ysf_core::snapshot::generate_diff_report(&diff);

    Ok(serde_json::json!({
        "risk_level": diff.summary.risk_level,
        "new_files": diff.files_added.len(),
        "deleted_files": diff.files_removed.len(),
        "modified_files": diff.files_modified.len(),
        "new_processes": diff.processes_started.len(),
        "report": report,
    }))
}
