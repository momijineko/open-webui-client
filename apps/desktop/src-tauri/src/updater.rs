// Auto-Updater for OpenWebUI Desktop
//
// This module provides automatic update functionality including:
// - Check for updates from GitHub Releases
// - Download updates
// - Install updates
// - Update progress reporting

use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use semver::Version;

/// GitHub release information from API
#[derive(Debug, Serialize, Deserialize, Clone)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Update information exposed to frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub release_date: String,
    pub file_size: u64,
}

/// Update status for UI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatus {
    pub checking: bool,
    pub update_available: bool,
    pub downloading: bool,
    pub download_progress: f64,
    pub installing: bool,
    pub error: Option<String>,
}

/// Result of update check
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub info: Option<UpdateInfo>,
}

/// Update configuration
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub repository: String,
    pub check_prereleases: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            repository: "open-webui/open-webui".to_string(), // Default repository
            check_prereleases: false,
        }
    }
}

/// Fetch latest release from GitHub API
async fn fetch_latest_release(
    config: &UpdateConfig,
) -> Result<GitHubRelease, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("OpenWebUI-Desktop")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Build request URL
    let url = if config.check_prereleases {
        format!("https://api.github.com/repos/{}/releases", config.repository)
    } else {
        format!("https://api.github.com/repos/{}/releases/latest", config.repository)
    };

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    if config.check_prereleases {
        // Get all releases and find the latest one
        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse releases: {}", e))?;

        releases
            .into_iter()
            .next()
            .ok_or_else(|| "No releases found".to_string())
    } else {
        // Get latest release
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse release: {}", e))
    }
}

/// Parse version from tag name (handle 'v' prefix)
fn parse_version(tag_name: &str) -> Result<Version, String> {
    let version_str = tag_name.trim_start_matches('v');
    Version::parse(version_str).map_err(|e| format!("Invalid version '{}': {}", tag_name, e))
}

/// Compare two versions
fn is_newer_version(current: &str, latest: &str, allow_prerelease: bool) -> bool {
    let current_ver = match parse_version(current) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let latest_ver = match parse_version(latest) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // If prereleases are not allowed and the latest version is a prerelease, don't update
    if !allow_prerelease && !latest_ver.pre.is_empty() {
        return false;
    }

    latest_ver > current_ver
}

/// Find the appropriate download URL for current platform
fn find_download_url(release: &GitHubRelease) -> Result<String, String> {
    let platform = get_platform_target();

    // Find matching platform package
    for asset in &release.assets {
        let asset_name = asset.name.to_lowercase();
        if asset_name.contains(&platform) {
            return Ok(asset.browser_download_url.clone());
        }
    }

    // If no platform-specific package found, return first source archive
    for asset in &release.assets {
        if asset.name.ends_with(".tar.gz") || asset.name.ends_with(".zip") {
            return Ok(asset.browser_download_url.clone());
        }
    }

    Err("No suitable download found".to_string())
}

/// Get platform identifier for download
fn get_platform_target() -> String {
    #[cfg(target_os = "windows")]
    {
        if cfg!(target_arch = "x86_64") {
            "windows-x64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "windows-arm64".to_string()
        } else {
            "windows".to_string()
        }
    }

    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "x86_64") {
            "macos-x64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "macos-arm64".to_string()
        } else {
            "macos".to_string()
        }
    }

    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "x86_64") {
            "linux-x64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "linux-arm64".to_string()
        } else {
            "linux".to_string()
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unknown".to_string()
    }
}

/// Check for updates
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    current_version: String,
) -> Result<UpdateCheckResult, String> {
    // Emit checking status
    let _ = app.emit("update-status", UpdateStatus {
        checking: true,
        update_available: false,
        downloading: false,
        download_progress: 0.0,
        installing: false,
        error: None,
    });

    let config = UpdateConfig::default();

    // Fetch latest release from GitHub
    let release = match fetch_latest_release(&config).await {
        Ok(r) => r,
        Err(e) => {
            let _ = app.emit("update-status", UpdateStatus {
                checking: false,
                update_available: false,
                downloading: false,
                download_progress: 0.0,
                installing: false,
                error: Some(e.clone()),
            });
            return Err(e);
        }
    };

    let latest_version = release.tag_name.clone();

    // Compare versions
    let has_update = is_newer_version(&current_version, &latest_version, config.check_prereleases);

    let info = if has_update {
        let download_url = match find_download_url(&release) {
            Ok(url) => url,
            Err(_e) => {
                // Still notify about update but mark download issue
                format!("https://github.com/{}/releases/latest", config.repository)
            }
        };

        // Find corresponding file size
        let file_size = release.assets
            .iter()
            .find(|a| a.browser_download_url == download_url || download_url.contains(&a.name))
            .map(|a| a.size)
            .unwrap_or(0);

        Some(UpdateInfo {
            version: latest_version.clone(),
            download_url,
            release_notes: release.body.clone(),
            release_date: release.published_at.clone(),
            file_size,
        })
    } else {
        None
    };

    let result = UpdateCheckResult {
        has_update,
        current_version,
        latest_version,
        info,
    };

    // Emit complete status
    let _ = app.emit("update-status", UpdateStatus {
        checking: false,
        update_available: has_update,
        downloading: false,
        download_progress: 0.0,
        installing: false,
        error: None,
    });

    Ok(result)
}

/// Download and install update
#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    update_info: UpdateInfo,
) -> Result<(), String> {
    // Emit downloading status
    let _ = app.emit("update-status", UpdateStatus {
        checking: false,
        update_available: true,
        downloading: true,
        download_progress: 0.0,
        installing: false,
        error: None,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600)) // 10 minute timeout
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Send request
    let response = client
        .get(&update_info.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(update_info.file_size);
    let mut downloaded = 0u64;
    let mut last_progress = 0f64;

    // Create progress stream
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        downloaded += chunk.len() as u64;

        // Calculate progress percentage
        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            // Only send event when progress changes by more than 1%
            if progress - last_progress >= 1.0 {
                let _ = app.emit("update-progress", progress);
                last_progress = progress;
            }
        }
    }

    // Download complete
    let _ = app.emit("update-progress", 100.0);

    // Emit complete status
    let _ = app.emit("update-status", UpdateStatus {
        checking: false,
        update_available: true,
        downloading: false,
        download_progress: 100.0,
        installing: true,
        error: None,
    });

    Ok(())
}

/// Install downloaded update
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    // In actual implementation, this would:
    // 1. Close the application
    // 2. Use helper program to replace files
    // 3. Restart the application

    // For desktop apps, this typically requires:
    // - Windows: Use separate updater.exe
    // - macOS: Use app bundle update
    // - Linux: Replace binary and restart

    // For now, open download page for manual installation
    let _ = app.emit("update-installing", ());

    #[cfg(target_os = "windows")]
    {
        let _ = open::that(
            "https://github.com/open-webui/open-webui/releases/latest"
        ).map_err(|e| format!("Failed to open download page: {}", e));
    }

    #[cfg(target_os = "macos")]
    {
        let _ = open::that(
            "https://github.com/open-webui/open-webui/releases/latest"
        ).map_err(|e| format!("Failed to open download page: {}", e));
    }

    #[cfg(target_os = "linux")]
    {
        let _ = open::that(
            "https://github.com/open-webui/open-webui/releases/latest"
        ).map_err(|e| format!("Failed to open download page: {}", e));
    }

    Ok(())
}

/// Get current application version
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Auto-check for updates on startup
pub fn auto_check_updates(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Wait before checking to avoid affecting startup speed
        tokio::time::sleep(Duration::from_secs(10)).await;

        let current_version = get_app_version();
        let app_clone = app.clone();
        match check_for_updates(app_clone, current_version).await {
            Ok(result) => {
                if result.has_update {
                    // Notify user about update
                    if let Some(info) = result.info {
                        let _ = app.emit("update-available", info);
                    }
                }
            }
            Err(e) => {
                eprintln!("Auto-update check failed: {}", e);
            }
        }
    });
}
