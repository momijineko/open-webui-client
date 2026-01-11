// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod config;
mod download;
mod instances;
mod proxy;
mod python_installer;

use backend::BackendState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(BackendState::new())
        .invoke_handler(tauri::generate_handler![
            // Backend commands
            backend::start_backend,
            backend::stop_backend,
            backend::check_backend_status,
            backend::check_backend_installation,
            backend::get_backend_logs,
            // Download commands
            download::install_local_backend,
            download::start_download,
            download::cancel_download,
            download::get_download_dir_path,
            // Python installer commands
            python_installer::install_python,
            python_installer::get_installed_python,
            python_installer::check_python_needed,
            // Instance commands
            instances::add_instance,
            instances::remove_instance,
            instances::get_instances,
            instances::connect_to_instance,
            instances::check_instance_status,
            // Config commands
            config::get_app_config,
            config::update_app_config,
            config::set_setup_completed,
            // Proxy commands
            proxy::proxy_request,
            proxy::fetch_image,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
