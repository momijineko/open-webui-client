// System Tray Integration for OpenWebUI Desktop
//
// This module provides system tray functionality including:
// - Tray icon with menu
// - Show/Hide window
// - Start/Stop backend
// - Quit application

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, Emitter,
};

/// Create the system tray with menu
pub fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let backend_start_item = MenuItem::with_id(app, "backend_start", "启动后端", true, None::<&str>)?;
    let backend_stop_item = MenuItem::with_id(app, "backend_stop", "停止后端", true, None::<&str>)?;
    let backend_status_item = MenuItem::with_id(app, "backend_status", "后端: 未运行", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = PredefinedMenuItem::quit(app, Some("退出"))?;

    // Create main menu
    let menu = Menu::with_items(app, &[
        &show_item,
        &hide_item,
        &separator1,
        &backend_start_item,
        &backend_stop_item,
        &backend_status_item,
        &separator2,
        &quit_item,
    ])?;

    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("OpenWebUI")
        .build(app)?;

    // Listen for tray events
    app.on_tray_icon_event(move |app, event| {
        match event {
            TrayIconEvent::Click {
                id: _,
                position: _,
                rect: _,
                button: _,
                button_state: _,
            } => {
                // Toggle window visibility on click
                // We'll handle this through menu items instead
            }
            TrayIconEvent::DoubleClick {
                id: _,
                position: _,
                rect: _,
                button: _,
            } => {
                // Show window on double click
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        }
    });

    // Listen for menu events
    app.on_menu_event(move |app, event| {
        match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "backend_start" => {
                // Emit event to frontend to start backend
                let _ = app.emit("tray-backend-start", ());
            }
            "backend_stop" => {
                // Emit event to frontend to stop backend
                let _ = app.emit("tray-backend-stop", ());
            }
            "quit" => {
                // Exit the application
                app.exit(0);
            }
            _ => {}
        }
    });

    Ok(())
}

/// Update the backend status in the tray menu
pub fn update_backend_status(app: &tauri::App, running: bool) -> Result<(), Box<dyn std::error::Error>> {
    let _status_text = if running { "后端: 运行中" } else { "后端: 未运行" };

    // Update the menu item text
    // Note: Tauri v2 doesn't have a direct API to update menu item text
    // This would require rebuilding the menu, which is a limitation
    // For now, we'll emit an event that the frontend can listen to
    let _ = app.emit("backend-status-changed", running);

    Ok(())
}
