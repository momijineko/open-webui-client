// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;
mod config;
mod download;
mod instances;
mod proxy;
mod python_installer;
mod tray;
mod updater;

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
            backend::initialize_user_backend,
            backend::update_user_backend,
            backend::get_backend_version_info,
            backend::check_and_auto_update_backend,
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
            // Updater commands
            updater::check_for_updates,
            updater::download_update,
            updater::install_update,
            updater::get_app_version,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            // Initialize system tray
            match tray::create_tray(app) {
                Ok(_) => println!("System tray initialized successfully"),
                Err(e) => eprintln!("Failed to initialize system tray: {}", e),
            }

            // Start auto-update check in background
            updater::auto_check_updates(app.handle().clone());

            // Auto-check and update backend on app startup
            println!("App started, checking backend updates...");
            match backend::check_and_auto_update_backend() {
                Ok(result) => {
                    println!("Backend check result: updated={}, message={}", result.updated, result.message);
                }
                Err(e) => {
                    eprintln!("Backend check failed: {}", e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
