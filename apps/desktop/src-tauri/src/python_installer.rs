use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Python installation progress
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PythonInstallProgress {
    pub current: u64,
    pub total: u64,
    pub file: String,
    pub speed: String,
    pub percentage: f64,
    pub stage: String, // "downloading", "extracting", "configuring"
}

/// Python installation result
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonInstallResult {
    pub success: bool,
    pub python_path: String,
    pub version: String,
    pub message: String,
}

/// Get Python data directory
fn get_python_data_dir() -> Result<PathBuf, String> {
    let dirs = directories::UserDirs::new()
        .ok_or("Failed to get user directories".to_string())?;

    let home_dir = dirs.home_dir();
    let python_dir = home_dir.join(".open-webui").join("python");

    Ok(python_dir)
}

/// Get platform-specific Python download URLs (use full ZIP package with pip)
fn get_python_download_urls(version: &str) -> Vec<String> {
    let mut urls = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if cfg!(target_arch = "x86_64") {
            // Use full ZIP package (non-embedded version)
            // Full version format: python-3.11.9-amd64.zip (includes pip)
            // Embedded version format: python-3.11.9-embed-amd64.zip (no pip)
            let full_tsinghua = format!(
                "https://mirrors.tuna.tsinghua.edu.cn/python/{}/python-{}-amd64.zip",
                version, version
            );
            let full_official = format!(
                "https://www.python.org/ftp/python/{}/python-{}-amd64.zip",
                version, version
            );

            urls.push(full_tsinghua);
            urls.push(full_official);
        } else if cfg!(target_arch = "aarch64") {
            let full = format!(
                "https://www.python.org/ftp/python/{}/python-{}-arm64.zip",
                version, version
            );
            urls.push(full);
        } else {
            let full = format!(
                "https://www.python.org/ftp/python/{}/python-{}-win32.zip",
                version, version
            );
            urls.push(full);
        }
    }

    #[cfg(target_os = "macos")]
    {
        urls.push(format!(
            "https://www.python.org/ftp/python/{}/python-{}-macos11.pkg",
            version, version
        ));
    }

    #[cfg(target_os = "linux")]
    {
        urls.push(format!(
            "https://www.python.org/ftp/python/{}/Python-{}.tgz",
            version, version
        ));
    }

    urls
}

/// Get platform-specific Python download URL (maintain backward compatibility)
fn get_python_download_url(version: &str) -> String {
    get_python_download_urls(version).first().cloned().unwrap_or_default()
}

/// Download file with progress reporting (retry mechanism)
async fn download_file_with_progress(
    url: &str,
    destination: &PathBuf,
    app_handle: &AppHandle,
    stage_name: &str,
    proxy_url: Option<&str>,
) -> Result<(), String> {
    let max_retries = 3;
    let mut last_error = String::new();

    for retry in 0..max_retries {
        if retry > 0 {
            eprintln!("Retrying download ({}/{}): {}", retry, max_retries, url);
            let _ = app_handle.emit("download-status", serde_json::json!({
                "status": format!("Retrying download ({}/{})...", retry, max_retries),
                "url": url
            }));
        }

        match download_file_once(url, destination, app_handle, stage_name, proxy_url).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("Download failed (attempt {}/{}): {}", retry + 1, max_retries, e);
                last_error = e;
            }
        }
    }

    Err(format!("Download failed after {} retries: {}", max_retries, last_error))
}

/// Single download attempt
async fn download_file_once(
    url: &str,
    destination: &PathBuf,
    app_handle: &AppHandle,
    stage_name: &str,
    proxy_url: Option<&str>,
) -> Result<(), String> {
    eprintln!("Starting download: {} -> {:?}", url, destination);

    // Create HTTP client with proxy support and longer timeout
    let client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 minute timeout
        .connect_timeout(std::time::Duration::from_secs(30));

    let client = if let Some(proxy) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;
        client_builder.proxy(proxy).build()
    } else {
        client_builder.build()
    }.map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "*/*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Referer", "https://mirrors.tuna.tsinghua.edu.cn/")
        .send()
        .await
        .map_err(|e| {
            eprintln!("Download request failed: {}", e);
            format!("Failed to fetch URL: {}", e)
        })?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // content_length may be None
    let total_size = response.content_length().unwrap_or(0);

    let mut downloaded = 0u64;
    let file = File::create(destination)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let mut bytes = response.bytes_stream();
    let start_time = std::time::Instant::now();

    use futures_util::StreamExt;

    while let Some(chunk_result) = bytes.next().await {
        let chunk = chunk_result
            .map_err(|e| format!("Failed to read chunk: {}", e))?;

        writer.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        // Calculate download speed
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        // Calculate percentage (if total size is known)
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // Send progress update
        let progress = PythonInstallProgress {
            current: downloaded,
            total: total_size.max(1), // Avoid division by zero
            file: destination.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("download")
                .to_string(),
            speed: format_speed(speed),
            percentage,
            stage: stage_name.to_string(),
        };

        let _ = app_handle.emit("download-progress", &progress);
    }

    writer.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    eprintln!("Download complete: {:?}", destination);
    Ok(())
}

/// Format download speed
fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec < 1024 {
        format!("{} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024 * 1024 {
        format!("{:.2} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{:.2} MB/s", bytes_per_sec as f64 / (1024.0 * 1024.0))
    }
}

/// Extract ZIP file
fn extract_zip(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let file = File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to get file {}: {}", i, e))?;

        let filepath = dest_dir.join(file.mangled_name().to_path_buf());

        if file.name().ends_with('/') {
            fs::create_dir_all(&filepath)
                .map_err(|e| format!("Failed to create directory {:?}: {}", filepath, e))?;
        } else {
            if let Some(parent) = filepath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory {:?}: {}", parent, e))?;
            }

            let mut outfile = File::create(&filepath)
                .map_err(|e| format!("Failed to create file {:?}: {}", filepath, e))?;

            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file {:?}: {}", filepath, e))?;
        }
    }

    Ok(())
}

/// Pip installation configuration
#[derive(Debug, Clone, Default)]
pub struct PipInstallConfig {
    pub pypi_mirror: Option<String>,
    pub proxy_url: Option<String>,
}

// get-pip.py script content (built-in, avoid network download)
// Fixed version: directly extract wheel to site-packages, don't depend on pip itself
const GET_PIP_SCRIPT: &str = r#"#!/usr/bin/env python3
#
# Fixed get-pip implementation for embedded Python
# Directly extracts pip wheel to site-packages without using pip itself
#

import os
import sys
import tempfile
import urllib.request
import json
import zipfile

def main():
    # PyPI simple API - get pip package info
    pypi_url = "https://pypi.org/pypi/pip/json"

    # Check for index-url from command line
    index_url = None
    for i, arg in enumerate(sys.argv):
        if arg == "--index-url" and i + 1 < len(sys.argv):
            index_url = sys.argv[i + 1]
            break

    # If index-url is provided, use it to find pip
    if index_url:
        # Parse the base URL from index-url
        base_url = index_url.replace("/simple", "").replace("/simple/", "").rstrip("/")
        pip_url = f"{base_url}/pypi/pip/json"
    else:
        pip_url = pypi_url

    try:
        # Download pip metadata
        print(f"Fetching pip info from: {pip_url}")
        with urllib.request.urlopen(pip_url, timeout=30) as response:
            data = json.loads(response.read().decode())

        # Get latest version
        version = data["info"]["version"]
        print(f"Latest pip version: {version}")

        # Find appropriate wheel for current platform
        urls = data["releases"][version]
        wheel_url = None

        # Priority: py3-none-any (universal) > platform-specific
        for url_info in urls:
            url = url_info["url"]
            filename = url_info["filename"]
            if "py3-none-any" in filename:
                wheel_url = url
                break

        if not wheel_url and urls:
            wheel_url = urls[0]["url"]

        if not wheel_url:
            print("Error: No suitable pip wheel found")
            sys.exit(1)

        print(f"Downloading pip from: {wheel_url}")

        # Download wheel
        with tempfile.NamedTemporaryFile(delete=False, suffix=".whl") as tmp_file:
            with urllib.request.urlopen(wheel_url, timeout=60) as response:
                tmp_file.write(response.read())
            wheel_path = tmp_file.name

        print(f"Installing pip from: {wheel_path}")

        # Find site-packages directory
        site_packages = os.path.join(os.path.dirname(sys.executable), "Lib", "site-packages")
        if not os.path.exists(site_packages):
            site_packages = os.path.join(sys.prefix, "Lib", "site-packages")
        if not os.path.exists(site_packages):
            # Fallback: create site-packages
            site_packages = os.path.join(os.path.dirname(sys.executable), "Lib", "site-packages")
            os.makedirs(site_packages, exist_ok=True)

        print(f"Extracting pip to: {site_packages}")

        # Extract wheel directly to site-packages
        with zipfile.ZipFile(wheel_path, 'r') as zip_ref:
            zip_ref.extractall(site_packages)

        print(f"pip successfully installed to: {site_packages}")

        # Clean up
        os.unlink(wheel_path)
        return 0

    except Exception as e:
        print(f"Error installing pip: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main() or 0)
"#;

/// Install pip to embedded Python
pub async fn install_pip(
    python_dir: &PathBuf,
    python_path: &str,
    _app_handle: &AppHandle,
    config: Option<&PipInstallConfig>,
) -> Result<(), String> {
    eprintln!("Attempting to use ensurepip to install pip...");

    // First try using ensurepip (if available)
    let ensurepip_output = std::process::Command::new(python_path)
        .args(["-m", "ensurepip", "--default-pip", "--upgrade"])
        .env("PYTHONPATH", python_dir.join("Lib").join("site-packages"))
        .output();

    if let Ok(output) = ensurepip_output {
        if output.status.success() {
            eprintln!("ensurepip installation successful");
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("ensurepip stdout: {}", stdout);
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("ensurepip failed: {}", stderr);
        }
    }

    // ensurepip not available, use built-in get-pip.py
    eprintln!("ensurepip not available, using built-in get-pip.py...");

    let get_pip_path = python_dir.join("get-pip.py");

    // Write built-in get-pip.py script
    fs::write(&get_pip_path, GET_PIP_SCRIPT)
        .map_err(|e| format!("Failed to write get-pip.py: {}", e))?;

    eprintln!("get-pip.py script written, attempting installation...");

    // Get configured PyPI mirror
    let pypi_mirror_url = config.and_then(|cfg| cfg.pypi_mirror.as_ref());

    let mut cmd = std::process::Command::new(python_path);
    cmd.arg(&get_pip_path)
        .env("PYTHONPATH", python_dir.join("Lib").join("site-packages"));

    // If PyPI mirror configured, add index-url parameter
    if let Some(mirror) = &pypi_mirror_url {
        eprintln!("Using PyPI mirror to install pip: {}", mirror);
        cmd.args(["--index-url", mirror]);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run get-pip.py: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("get-pip.py stdout: {}", stdout);
    if !stderr.is_empty() {
        eprintln!("get-pip.py stderr: {}", stderr);
    }

    if output.status.success() {
        eprintln!("pip installation successful");

        // Clean up get-pip.py
        let _ = fs::remove_file(&get_pip_path);

        // For Windows embedded Python, need to reconfigure .pth file
        #[cfg(target_os = "windows")]
        {
            eprintln!("Reconfiguring Python .pth file to enable pip...");
            if let Err(e) = configure_embedded_python(python_dir) {
                eprintln!("Warning: Failed to configure .pth file: {}", e);
            }

            // Extra measure: Create pip.pth file in site-packages
            // This way pip can be found even if main .pth file doesn't work
            let site_packages = python_dir.join("Lib").join("site-packages");
            let pip_pth = site_packages.join("pip.pth");

            // Create a .pth file pointing to pip package
            // This tells Python to add pip directory to sys.path
            let pip_pth_content = format!("import site; site.addsitedir(r'{}')", site_packages.to_string_lossy().replace('\\', "/"));
            if let Err(e) = fs::write(&pip_pth, pip_pth_content) {
                eprintln!("Warning: Failed to create pip.pth: {}", e);
            } else {
                eprintln!("Created pip.pth: {:?}", pip_pth);
            }

            // Debug: List site-packages directory contents
            eprintln!("Checking site-packages directory: {:?}", site_packages);
            if site_packages.exists() {
                if let Ok(entries) = fs::read_dir(&site_packages) {
                    for entry in entries.flatten() {
                        eprintln!("  - {:?}", entry.file_name());
                    }
                }
            } else {
                eprintln!("site-packages directory does not exist!");
            }

            // Debug: Read .pth file contents
            if let Ok(entries) = fs::read_dir(python_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("pth") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            eprintln!(".pth file {:?} content:\n{}", path, content);
                        }
                    }
                }
            }
        }

        // Verify pip is really available (using PYTHONPATH)
        eprintln!("Verifying pip installation...");
        let site_packages = python_dir.join("Lib").join("site-packages");
        let site_packages_path = site_packages.to_string_lossy().replace('\\', "/");

        let verify_code = format!(
            "import sys; sys.path.insert(0, r'{}'); import pip; print('pip import OK')",
            site_packages_path
        );

        let verify_output = std::process::Command::new(python_path)
            .args(["-c", &verify_code])
            .output();

        match verify_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("pip verification - stdout: {}, stderr: {}", stdout, stderr);
                if !output.status.success() {
                    eprintln!("Warning: pip module cannot be imported");
                }
            }
            Err(e) => {
                eprintln!("pip verification failed: {}", e);
            }
        }

        Ok(())
    } else {
        // Clean up get-pip.py
        let _ = fs::remove_file(&get_pip_path);
        Err(format!("get-pip.py failed:\nstdout: {}\nstderr: {}", stdout, stderr))
    }
}

/// Configure embedded Python (Windows)
#[cfg(target_os = "windows")]
fn configure_embedded_python(python_dir: &PathBuf) -> Result<(), String> {
    eprintln!("configure_embedded_python called with: {:?}", python_dir);

    // Create site-packages directory
    let site_packages = python_dir.join("Lib").join("site-packages");
    fs::create_dir_all(&site_packages)
        .map_err(|e| format!("Failed to create site-packages: {}", e))?;

    eprintln!("site-packages directory: {:?}", site_packages);

    // Modify python3xx._pth file to enable site-packages
    let pth_files = fs::read_dir(python_dir)
        .map_err(|e| format!("Failed to read python dir: {}", e))?;

    let mut found_pth = false;
    for entry in pth_files {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let path = entry.path();

        eprintln!("Checking file: {:?}", path);

        if path.extension().and_then(|s| s.to_str()) == Some("pth") {
            found_pth = true;
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read pth file: {}", e))?;

            eprintln!("Original .pth file content:\n{}", content);

            let mut new_content = content.clone();

            // Uncomment import site (this enables site.py module)
            if content.contains("# import site") {
                new_content = new_content.replace("# import site", "import site");
                eprintln!("Enabled 'import site'");
            }

            // Add site-packages using absolute path
            // Windows .pth files need forward slash or double backslash
            let site_packages_str = site_packages.to_string_lossy().replace('\\', "/");

            if !content.contains("site-packages") && !content.contains(&site_packages_str) {
                new_content = format!("{}\n{}\n", new_content.trim(), site_packages_str);
                eprintln!("Added site-packages path: {}", site_packages_str);
            }

            eprintln!("New .pth file content:\n{}", new_content);

            fs::write(&path, new_content)
                .map_err(|e| format!("Failed to write pth file: {}", e))?;

            eprintln!("Configured .pth file: {:?}", path);
        }
    }

    if !found_pth {
        eprintln!("Warning: .pth file not found!");
    }

    Ok(())
}

/// Check if Python is installed
pub fn check_python_installed(python_dir: &PathBuf) -> Option<String> {
    #[cfg(target_os = "windows")]
    let python_exe = python_dir.join("python.exe");

    #[cfg(not(target_os = "windows"))]
    let python_exe = python_dir.join("bin").join("python3");

    if python_exe.exists() {
        python_exe.to_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if pip is installed
pub fn check_pip_installed(python_path: &str) -> bool {
    let mut cmd = create_python_command_with_env(python_path);
    cmd.args(["-m", "pip", "--version"]);

    if let Ok(output) = cmd.output() {
        output.status.success()
    } else {
        false
    }
}

/// Install Python (use full ZIP package with pip)
#[tauri::command]
pub async fn install_python(app_handle: AppHandle) -> Result<PythonInstallResult, String> {
    eprintln!("Starting Python installation...");

    let python_dir = get_python_data_dir()?;
    eprintln!("Python data directory: {:?}", python_dir);

    #[cfg(target_os = "windows")]
    let runtime_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let runtime_dir = python_dir.join("runtime");

    // Check if already installed
    if let Some(python_path) = check_python_installed(&runtime_dir) {
        eprintln!("Python Already installed: {:?}", python_path);

        // Full Python comes with pip, but verify
        if !check_pip_installed(&python_path) {
            eprintln!("Warning: Python installed but pip not available");
        }

        return Ok(PythonInstallResult {
            success: true,
            python_path,
            version: "3.11".to_string(),
            message: "Python already installed".to_string(),
        });
    }

    // Create directory
    fs::create_dir_all(&python_dir)
        .map_err(|e| format!("Failed to create python directory: {}", e))?;

    let python_version = "3.11.9"; // Use stable Python version
    let download_urls = get_python_download_urls(python_version);

    eprintln!("Downloading Python from {} mirror sources", download_urls.len());

    // Download path
    let zip_filename = format!("python-{}.zip", python_version);
    let zip_path = python_dir.join(&zip_filename);

    // Try downloading from multiple mirrors
    let mut download_success = false;
    let mut last_error = String::new();

    for (idx, download_url) in download_urls.iter().enumerate() {
        eprintln!("Attempting mirror {}/{}: {}", idx + 1, download_urls.len(), download_url);

        // Send start download event
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": format!("Downloading Python ({}/{})...", idx + 1, download_urls.len()),
            "url": download_url
        }));

        match download_file_with_progress(download_url, &zip_path, &app_handle, "downloading", None).await {
            Ok(()) => {
                eprintln!("Download successful: {}", download_url);
                download_success = true;
                break;
            }
            Err(e) => {
                eprintln!("Download from mirror {} failed: {}", download_url, e);
                last_error = e;
                // Delete partially downloaded file
                let _ = fs::remove_file(&zip_path);
            }
        }
    }

    if !download_success {
        return Err(format!("All mirrors failed. Last error: {}", last_error));
    }

    eprintln!("Download complete, starting extraction...");

    // Send extraction event
    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "Extracting...",
        "url": download_urls.first().unwrap_or(&String::new())
    }));

    // Extract Python
    #[cfg(target_os = "windows")]
    let extract_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let extract_dir = python_dir.join("runtime");

    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    extract_zip(&zip_path, &extract_dir)?;

    eprintln!("Extraction complete");

    // Clean up downloaded zip file
    let _ = fs::remove_file(&zip_path);

    // Find Python executable
    // Full ZIP may have different directory structure
    let python_path = find_python_in_dir(&extract_dir)
        .ok_or("Python executable not found after installation".to_string())?;

    eprintln!("Python installation successful: {:?}", python_path);

    // Verify pip is available (full version should have it)
    eprintln!("Verifying pip availability...");
    if check_pip_installed(&python_path) {
        eprintln!("pip is ready");
    } else {
        eprintln!("Warning: pip not available, may need manual installation");
    }

    // Send completion event
    let _ = app_handle.emit("download-complete", serde_json::json!({
        "python_path": python_path,
        "version": python_version
    }));

    Ok(PythonInstallResult {
        success: true,
        python_path: python_path.clone(),
        version: python_version.to_string(),
        message: "Python installed successfully".to_string(),
    })
}

/// Find Python executable in directory
fn find_python_in_dir(dir: &PathBuf) -> Option<String> {
    // First try to find directly in the directory
    #[cfg(target_os = "windows")]
    let direct_exe = dir.join("python.exe");

    #[cfg(not(target_os = "windows"))]
    let direct_exe = dir.join("bin/python3");

    if direct_exe.exists() {
        return direct_exe.to_str().map(|s| s.to_string());
    }

    // Try to find in subdirectories (full ZIP may have nested directories)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                #[cfg(target_os = "windows")]
                let exe_path = path.join("python.exe");

                #[cfg(not(target_os = "windows"))]
                let exe_path = path.join("bin/python3");

                if exe_path.exists() {
                    return exe_path.to_str().map(|s| s.to_string());
                }
            }
        }
    }

    None
}

/// Get installed Python path
#[tauri::command]
pub fn get_installed_python() -> Result<Option<String>, String> {
    let python_dir = get_python_data_dir()?;

    #[cfg(target_os = "windows")]
    let runtime_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let runtime_dir = python_dir.join("runtime");

    if let Some(python_path) = check_python_installed(&runtime_dir) {
        Ok(Some(python_path))
    } else {
        Ok(None)
    }
}

/// Check if Python needs to be installed
#[tauri::command]
pub fn check_python_needed() -> Result<bool, String> {
    // First check system Python
    if find_system_python().is_some() {
        return Ok(false);
    }

    // Check app-embedded Python
    let python_dir = get_python_data_dir()?;

    #[cfg(target_os = "windows")]
    let runtime_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let runtime_dir = python_dir.join("runtime");

    Ok(check_python_installed(&runtime_dir).is_none())
}

/// Find system Python
fn find_system_python() -> Option<String> {
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = std::process::Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if version_str.contains("Python 3.") {
                    return Some(cmd.to_string());
                }
            }
        }
    }

    None
}

/// Create command with correct environment variables for embedded Python
/// Full Python doesn't need special environment variables
pub fn create_python_command_with_env(python_path: &str) -> std::process::Command {
    std::process::Command::new(python_path)
}

/// Get Python site-packages path
pub fn get_site_packages_path(python_path: &str) -> Option<PathBuf> {
    if let Ok(python_path_buf) = std::path::PathBuf::from(python_path).canonicalize() {
        if let Some(python_dir) = python_path_buf.parent() {
            let site_packages = python_dir.join("Lib").join("site-packages");
            if site_packages.exists() {
                return Some(site_packages);
            }
        }
    }
    None
}
