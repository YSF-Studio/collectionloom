use tauri::Manager;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("ZipLoom — Archive Utility")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::supported_formats,
            commands::inspect_archive,
            commands::compress_files,
            commands::extract_archive,
            commands::about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZipLoom");
}
