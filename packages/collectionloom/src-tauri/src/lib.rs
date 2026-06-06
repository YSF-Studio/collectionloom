use tauri::Manager;

pub mod commands;
pub mod collector;
pub mod compare;
pub mod export;
pub mod models;
pub mod prd_commands;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("CollectionLoom — Portable Forensic Acquisition")?;

            // ─── GUI Screenshot Mode ───
            if std::env::var("COLLECTIONLOOM_SCREENSHOT").is_ok() {
                let w = window.clone();
                std::thread::spawn(move || {
                    use std::time::Duration;
                    std::thread::sleep(Duration::from_secs(5));

                    // Cycle through sidebar sections by text label
                    let sections = [
                        "RAM Capture", "Mobile Triage", "Cloud Snapshot",
                        "Network Capture", "System Snapshot", "Encryption",
                        "Hash Verify", "Custody Chain", "About", "Disk Imaging"
                    ];
                    for section in &sections {
                        let js = format!(
                            "Array.from(document.querySelectorAll('.sidebar-item')).find(b=>b.textContent.includes('{}'))?.click();",
                            section
                        );
                        let _ = w.eval(&js);
                        std::thread::sleep(Duration::from_secs(4));
                    }

                    // Final: back to disk
                    let _ = w.eval("Array.from(document.querySelectorAll('.sidebar-item')).find(b=>b.textContent.includes('Disk Imaging'))?.click();");
                    std::thread::sleep(Duration::from_secs(3));
                    eprintln!("[SCREENSHOT] All sections navigated");
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_disks,
            commands::start_disk_imaging,
            commands::get_imaging_progress,
            commands::cancel_imaging,
            commands::list_ram_tools,
            commands::get_ram_size,
            commands::list_processes,
            commands::capture_ram,
            commands::enable_write_blocker,
            commands::disable_write_blocker,
            commands::check_write_blocker,
            commands::list_android_devices,
            commands::adb_backup,
            commands::list_ios_devices,
            commands::ios_backup,
            commands::list_interfaces,
            commands::start_network_capture,
            commands::cancel_network_capture,
            commands::scan_encryption,
            commands::create_chain_of_custody,
            commands::generate_coc_report,
            commands::about_info,
            commands::take_snapshot,
            commands::compare_snapshot,
            commands::create_cloud_snapshot,
            commands::verify_hash,
            commands::sign_coc,
            commands::hpa_dco_detect,
            commands::generate_evidence_id,
            commands::compute_file_hash,
            prd_commands::create_case,
            prd_commands::list_cases_cmd,
            prd_commands::get_case,
            prd_commands::start_snapshot,
            prd_commands::get_snapshot_progress,
            prd_commands::list_snapshots_cmd,
            prd_commands::get_snapshot,
            prd_commands::compare_snapshots,
            prd_commands::list_diffs_cmd,
            prd_commands::export_json,
            prd_commands::export_markdown,
            prd_commands::export_zip,
            prd_commands::list_exports,
            prd_commands::generate_qr_label,
            prd_commands::get_capture_packets,
            prd_commands::get_capture_stats,
            prd_commands::list_case_summaries_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CollectionLoom");
}
