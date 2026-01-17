use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// Whether setup has been completed
    pub setup_completed: bool,
    /// Selected mode: 'local' or 'remote'
    pub setup_mode: Option<String>,
    /// For remote mode: configured server URL
    pub remote_url: Option<String>,
    /// For remote mode: username for Basic Auth
    pub remote_username: Option<String>,
    /// For remote mode: password for Basic Auth
    pub remote_password: Option<String>,
    /// Current backend version (for local mode)
    pub backend_version: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            setup_completed: false,
            setup_mode: None,
            remote_url: None,
            remote_username: None,
            remote_password: None,
            backend_version: None,
        }
    }
}

/// Get the config file path in user data directory
fn get_config_path() -> Result<PathBuf, String> {
    if let Some(dirs) = directories::UserDirs::new() {
        let home_dir = dirs.home_dir();
        let config_dir = home_dir.join(".open-webui");

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(config_dir.join("config.json"))
    } else {
        Err("Failed to get user home directory".to_string())
    }
}

/// Load app configuration from file
pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))
}

/// Save app configuration to file
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    load_config()
}

#[tauri::command]
pub fn update_app_config(config: AppConfig) -> Result<(), String> {
    save_config(&config)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_setup_completed(
    mode: String,
    remote_url: Option<String>,
    remote_username: Option<String>,
    remote_password: Option<String>,
) -> Result<(), String> {
    let mut config = load_config().unwrap_or_default();
    config.setup_completed = true;
    config.setup_mode = Some(mode);
    config.remote_url = remote_url;
    config.remote_username = remote_username;
    config.remote_password = remote_password;
    save_config(&config)
}
