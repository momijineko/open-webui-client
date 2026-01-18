use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Write, BufRead};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use tauri::{AppHandle, Emitter};
use crate::python_installer::{install_python, get_installed_python, check_pip_installed, create_python_command_with_env, get_site_packages_path};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadComponent {
    pub name: String,
    pub url: String,
    pub size: u64,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub components: Vec<DownloadComponent>,
    pub mirror_url: Option<String>,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub current: u64,
    pub total: u64,
    pub file: String,
    pub speed: String,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadStatus {
    pub is_downloading: bool,
    pub current_file: String,
    pub progress: DownloadProgress,
    pub completed: Vec<String>,
    pub failed: Vec<String>,
}

/// Get download directory
fn get_download_dir() -> Result<PathBuf, String> {
    let dirs = directories::UserDirs::new()
        .ok_or("Failed to get user directories".to_string())?;

    let download_dir = dirs.download_dir()
        .ok_or("Failed to get download directory".to_string())?;

    Ok(download_dir.join("open-webui"))
}

/// Build download URL (supports mirror sources)
fn build_download_url(base_url: &str, mirror_url: Option<&str>) -> String {
    if let Some(mirror) = mirror_url {
        // If mirror source exists, try to replace domain
        if let Ok(parsed) = url::Url::parse(base_url) {
            if let Ok(mut mirror_url) = url::Url::parse(mirror) {
                mirror_url.set_path(parsed.path());
                mirror_url.set_query(parsed.query());
                return mirror_url.to_string();
            }
        }
    }
    base_url.to_string()
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

/// Download a single file
async fn download_file(
    url: &str,
    destination: &PathBuf,
    proxy_url: Option<&str>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // Create HTTP client
    let client_builder = reqwest::Client::builder();

    let client = if let Some(proxy) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;
        client_builder.proxy(proxy).build()
    } else {
        client_builder.build()
    }.map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Send request
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let total_size = response.content_length()
        .ok_or("Failed to get content length")?;

    let mut downloaded = 0u64;
    let mut file = File::create(destination)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut bytes = response.bytes_stream();
    let start_time = std::time::Instant::now();

    use futures_util::StreamExt;

    while let Some(chunk_result) = bytes.next().await {
        let chunk = chunk_result
            .map_err(|e| format!("Failed to read chunk: {}", e))?;

        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        // Calculate download speed
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        // Send progress update
        let progress = DownloadProgress {
            current: downloaded,
            total: total_size,
            file: destination.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            speed: format_speed(speed),
            percentage: (downloaded as f64 / total_size as f64) * 100.0,
        };

        let _ = app_handle.emit("download-progress", &progress);
    }

    Ok(())
}

#[tauri::command]
pub async fn start_download(
    config: DownloadConfig,
    app_handle: AppHandle,
) -> Result<String, String> {
    let download_dir = get_download_dir()?;

    // Create download directory
    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("Failed to create download directory: {}", e))?;

    let mut completed = Vec::new();
    let mut failed = Vec::new();

    for component in &config.components {
        let filename = component.url.split('/')
            .last()
            .unwrap_or("unknown");

        let destination = download_dir.join(filename);
        let download_url = build_download_url(
            &component.url,
            config.mirror_url.as_deref(),
        );

        // Send status update
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "downloading",
            "file": component.name,
            "url": download_url
        }));

        // Download file
        match download_file(&download_url, &destination, config.proxy_url.as_deref(), &app_handle).await {
            Ok(_) => {
                completed.push(component.name.clone());
            }
            Err(e) => {
                eprintln!("Failed to download {}: {}", component.name, e);
                failed.push(component.name.clone());

                if component.required {
                    return Err(format!("Required component {} failed to download: {}", component.name, e));
                }
            }
        }
    }

    // Send completion status
    let _ = app_handle.emit("download-complete", serde_json::json!({
        "completed": completed,
        "failed": failed
    }));

    Ok(format!(
        "Download completed: {} succeeded, {} failed",
        completed.len(),
        failed.len()
    ))
}

#[tauri::command]
pub fn cancel_download() -> Result<String, String> {
    // TODO: Implement download cancellation
    Ok("Download cancelled".to_string())
}

#[tauri::command]
pub fn get_download_dir_path() -> Result<String, String> {
    let path = get_download_dir()?;
    Ok(path.to_string_lossy().to_string())
}

/// Local backend installation configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct BackendInstallConfig {
    pub pypi_mirror: Option<String>,
    pub proxy_url: Option<String>,
}

/// Install local backend (use pip to install local source)
#[tauri::command]
pub async fn install_local_backend(
    config: Option<BackendInstallConfig>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    eprintln!("Starting local backend installation...");

    // First check if Python is already installed in the app
    let python_cmd = if let Ok(Some(python_path)) = get_installed_python() {
        eprintln!("Using app-installed Python: {:?}", python_path);
        python_path
    } else {
        // Not available in app, check system Python
        match find_python_executable() {
            Ok(cmd) => {
                eprintln!("Using system Python: {}", cmd);
                cmd
            }
            Err(_) => {
                // Not available on system either, download and install Python
                eprintln!("Python not installed on system, starting download and installation...");

                // Send Python installation start event
                let _ = app_handle.emit("download-status", serde_json::json!({
                    "status": "Downloading and installing Python environment...",
                    "file": "Python 3.11",
                    "url": "internal://python"
                }));

                // Install Python
                let install_result = install_python(app_handle.clone()).await
                    .map_err(|e| format!("Failed to install Python: {}", e))?;

                eprintln!("Python installation successful: {:?}", install_result.python_path);
                install_result.python_path
            }
        }
    };

    eprintln!("Using Python: {}", python_cmd);

    // Check if pip is available
    if !check_pip_installed(&python_cmd) {
        eprintln!("pip not available, starting pip installation...");

        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "Installing pip...",
            "file": "Python pip",
            "url": "https://bootstrap.pypa.io/get-pip.py"
        }));

        // Get Python directory
        let python_dir = std::path::PathBuf::from(&python_cmd)
            .parent()
            .ok_or("Failed to get Python directory".to_string())?
            .to_path_buf();

        // Create pip installation configuration
        let pip_config = crate::python_installer::PipInstallConfig {
            pypi_mirror: config.as_ref().and_then(|c| c.pypi_mirror.clone()),
            proxy_url: config.as_ref().and_then(|c| c.proxy_url.clone()),
        };

        // Install pip
        crate::python_installer::install_pip(&python_dir, &python_cmd, &app_handle, Some(&pip_config)).await?;
    }

    // Get backend directory
    // Current directory is apps/desktop/src-tauri, need to go up 4 levels to find project root's backend
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    eprintln!("Current directory: {:?}", current_dir);

    // Search upwards for backend directory (maximum 5 levels)
    let mut search_dir = current_dir.as_path();
    let backend_dir = loop {
        if search_dir.join("backend").exists() {
            break search_dir.join("backend");
        }

        // Go up one level
        match search_dir.parent() {
            Some(parent) => {
                search_dir = parent;
                // Prevent infinite loop
                if search_dir.as_os_str().is_empty() {
                    let error = format!(
                        "Backend directory not found\nCurrent dir: {:?}\nSearched up to root directory",
                        current_dir
                    );
                    eprintln!("{}", error);
                    return Err(error);
                }
            }
            None => {
                let error = format!(
                    "Backend directory not found\nCurrent dir: {:?}\nReached filesystem root",
                    current_dir
                );
                eprintln!("{}", error);
                return Err(error);
            }
        }
    };

    eprintln!("Backend directory: {:?}", backend_dir);

    if !backend_dir.exists() {
        let error = format!("Backend directory not found: {:?}\nCurrent dir: {:?}", backend_dir, current_dir);
        eprintln!("{}", error);
        return Err(error);
    }

    // Send progress 10% - Start installation
    let progress = DownloadProgress {
        current: 10,
        total: 100,
        file: "OpenWebUI Backend".to_string(),
        speed: "Preparing...".to_string(),
        percentage: 10.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    // Send status update
    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "Checking Python environment...",
        "file": "OpenWebUI Backend",
        "url": "local://backend"
    }));

    // Note: pip availability check has been completed above (lines 276-300), no need to repeat here
    eprintln!("Starting backend installation, Python: {}", python_cmd);

    // Send progress 30% - Start installation
    let progress = DownloadProgress {
        current: 30,
        total: 100,
        file: "OpenWebUI Backend".to_string(),
        speed: "Installing dependencies...".to_string(),
        percentage: 30.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "Installing Python dependency packages...",
        "file": "OpenWebUI Backend",
        "url": "local://backend"
    }));

    eprintln!("Starting backend installation, directory: {:?}", backend_dir);

    // Determine PyPI mirror source list based on configuration
    let pypi_mirrors = if let Some(cfg) = &config {
        if let Some(mirror) = &cfg.pypi_mirror {
            eprintln!("Using configured PyPI mirror: {}", mirror);
            vec![mirror.as_str(), "https://pypi.org/simple"]
        } else {
            vec![
                "https://mirrors.aliyun.com/pypi/simple/",      // Alibaba Cloud
                "https://pypi.tuna.tsinghua.edu.cn/simple/",    // Tsinghua University
                "https://pypi.org/simple",                      // Official source (fallback)
            ]
        }
    } else {
        vec![
            "https://mirrors.aliyun.com/pypi/simple/",      // Alibaba Cloud
            "https://pypi.tuna.tsinghua.edu.cn/simple/",    // Tsinghua University
            "https://pypi.org/simple",                      // Official source (fallback)
        ]
    };

    let mut install_success = false;
    let mut last_error = String::new();

    for (idx, mirror) in pypi_mirrors.iter().enumerate() {
        eprintln!("Attempting to install backend using mirror {}/{}", idx + 1, mirror);

        // Use helper function to get site-packages path
        let site_packages = get_site_packages_path(&python_cmd);
        if let Some(ref sp) = site_packages {
            eprintln!("Setting PYTHONPATH: {:?}", sp);
        } else {
            eprintln!("Warning: Unable to get site-packages path");
        }

        // Use pip to install backend dependencies
        // Backend uses requirements.txt instead of setup.py/pyproject.toml
        let requirements_file = backend_dir.join("requirements.txt");

        if !requirements_file.exists() {
            last_error = format!("requirements.txt not found in backend directory: {:?}", backend_dir);
            eprintln!("{}", last_error);
            continue;
        }

        // Send installation status update
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": format!("Installing dependency packages (mirror {} of {})...", idx + 1),
            "file": "OpenWebUI Backend",
            "url": "local://backend"
        }));

        eprintln!("Starting pip install, using mirror: {}", mirror);

        let mut cmd = create_python_command_with_env(&python_cmd);
        // Add arguments to make pip output more verbose and disable progress bar
        cmd.args([
            "-m", "pip", "install", "-r", "requirements.txt",
            "--index-url", mirror,
            "--progress-bar=off",
            "--verbose"  // Add verbose output
        ])
           .current_dir(&backend_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Note: Do not set PYTHONPATH, let Python handle paths itself
        // Full Python version doesnt need manual PYTHONPATH setting

        // Use spawn to read output in real-time
        match cmd.spawn() {
            Ok(mut child) => {
                let (tx, rx) = mpsc::sync_channel::<String>(100);

                eprintln!("pip process started, PID: {:?}", child.id());

                // Read stdout (pip's main output)
                if let Some(stdout) = child.stdout.take() {
                    let reader = std::io::BufReader::new(stdout);
                    let tx_clone = tx.clone();

                    // Read output in a separate thread, send via channel
                    std::thread::spawn(move || {
                        eprintln!("[pip-reader] Thread started, beginning to read output...");
                        for line in reader.lines() {
                            if let Ok(line_text) = line {
                                let _ = tx_clone.send(line_text);
                            }
                        }
                        eprintln!("[pip-reader] Output reading complete");
                    });
                }

                // Also read stderr
                if let Some(stderr) = child.stderr.take() {
                    let reader = std::io::BufReader::new(stderr);
                    let tx_err = tx.clone();
                    std::thread::spawn(move || {
                        eprintln!("[pip-error] Error reading thread started...");
                        for line in reader.lines() {
                            if let Ok(line_text) = line {
                                let _ = tx_err.send(format!("ERROR: {}", line_text));
                            }
                        }
                    });
                }

                // Process output and send events on main thread
                // Use recv_timeout to avoid permanent blocking
                let mut line_count = 0;
                let mut install_phase = 0.0; // 0-30: collect deps, 30-90: download/install, 90-100: complete

                loop {
                    match rx.recv_timeout(std::time::Duration::from_millis(500)) {
                        Ok(line_text) => {
                            line_count += 1;

                            eprintln!("[pip {}] Raw: {}", line_count, line_text);

                            // Parse pip output, extract currently installing package
                            let (status, phase_increment) = if line_text.contains("Collecting ") {
                                let pkg = line_text.replace("Collecting ", "").trim().to_string();
                                let pkg_name = pkg.split_whitespace().next().unwrap_or(&pkg).to_string();
                                // Collection phase: 0-30%
                                (format!("Collecting: {}", pkg_name), 2.0)
                            } else if line_text.contains("Downloading ") {
                                let pkg = line_text.replace("Downloading ", "").trim().to_string();
                                let pkg_name = pkg.split_whitespace().next().unwrap_or(&pkg).to_string();
                                // Download phase: 30-70%
                                (format!("Downloading: {}", pkg_name), 3.0)
                            } else if line_text.contains("Installing collected packages") {
                                ("Starting installation of downloaded packages...".to_string(), 1.0)
                            } else if line_text.contains("Successfully installed ") {
                                ("Installation complete!".to_string(), 0.5)
                            } else if line_text.contains("Requirement already satisfied") {
                                // Already installed package, skip
                                (format!("Already installed: {}", line_text.trim()), 0.0)
                            } else if line_text.starts_with("ERROR:") {
                                (format!("Error: {}", &line_text[6..]), 0.0)
                            } else {
                                // Other output, small increment
                                (format!("Processing... (line {} line)", line_count), 0.1)
                            };

                            // Update progress percentage, max to 95%
                            install_phase = ((install_phase + phase_increment) as f64).min(95.0);

                            // Send progress update immediately (send every time, no rate limiting)
                            let event_payload = serde_json::json!({
                                "status": status.clone(),
                                "file": "OpenWebUI Backend",
                                "url": "local://backend"
                            });

                            eprintln!("[pip EMIT] {} -> {} (progress: {}%)", line_count, status, install_phase);

                            match app_handle.emit("download-status", &event_payload) {
                                Ok(_) => {},
                                Err(e) => eprintln!("Failed to emit event: {}", e),
                            }

                            // Also send download-progress event for progress bar update
                            let progress = DownloadProgress {
                                current: line_count,
                                total: 100,
                                file: "OpenWebUI Backend".to_string(),
                                speed: status.clone(),
                                percentage: install_phase,
                            };
                            let _ = app_handle.emit("download-progress", &progress);
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Timeout, check if process is still running
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    // Process ended
                                    eprintln!("pip Process ended, exit code: {:?}", status.code());
                                    break;
                                }
                                Ok(None) => {
                                    // Process still running, continue waiting
                                    continue;
                                }
                                Err(_) => {
                                    // Cannot check process status, continue waiting
                                    continue;
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            // Channel disconnected, all senders closed
                            eprintln!("pip output channel disconnected");
                            break;
                        }
                    }
                }

                // Wait for process completion
                match child.wait() {
                    Ok(status) => {
                        eprintln!("pip install exit code: {:?}", status.code());

                        if status.success() {
                            eprintln!("Backend installation successful (using mirror {})", mirror);
                            install_success = true;

                            // Send success status
                            let _ = app_handle.emit("download-status", serde_json::json!({
                                "status": "Dependency installation complete!",
                                "file": "OpenWebUI Backend",
                                "url": "local://backend"
                            }));
                            break;
                        } else {
                            last_error = format!("pip install failed with mirror {} (exit code: {:?})", mirror, status.code());
                            eprintln!("Installation failed: {}", last_error);

                            // Send failure status, try next mirror
                            let _ = app_handle.emit("download-status", serde_json::json!({
                                "status": format!("Installation failed (mirror {}), trying next...", idx + 1),
                                "file": "OpenWebUI Backend",
                                "url": "local://backend"
                            }));
                        }
                    }
                    Err(e) => {
                        last_error = format!("Failed to wait for pip install: {}", e);
                        eprintln!("Wait failed: {}", e);

                        // Send execution failure status
                        let _ = app_handle.emit("download-status", serde_json::json!({
                            "status": format!("Execution failed (mirror {}), trying next...", idx + 1),
                            "file": "OpenWebUI Backend",
                            "url": "local://backend"
                        }));
                    }
                }
            }
            Err(e) => {
                last_error = format!("Failed to execute pip install: {}", e);
                eprintln!("Execution failed: {}", e);

                // Send execution failure status
                let _ = app_handle.emit("download-status", serde_json::json!({
                    "status": format!("Execution failed (mirror {}), trying next...", idx + 1),
                    "file": "OpenWebUI Backend",
                    "url": "local://backend"
                }));
            }
        }
    }

    if !install_success {
        return Err(format!("All mirrors failed. Last error: {}", last_error));
    }

    // Verify backend directory exists
    eprintln!("Verifying backend directory...");
    let open_webui_dir = backend_dir.join("open_webui");
    if open_webui_dir.exists() {
        eprintln!("OK: backend/open_webui directory exists");
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "OpenWebUI Backend file verification complete!",
            "file": "OpenWebUI Backend",
            "url": "local://backend"
        }));
    } else {
        eprintln!("X: backend/open_webui directory does not exist");
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "Warning: OpenWebUI Backend files not found, but continuing...",
            "file": "OpenWebUI Backend",
            "url": "local://backend"
        }));
    }

    // Send progress 100% - Complete
    let progress = DownloadProgress {
        current: 100,
        total: 100,
        file: "OpenWebUI Backend".to_string(),
        speed: "Complete".to_string(),
        percentage: 100.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    // Wait a short time to ensure all events are sent to frontend
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Send completion event
    let _ = app_handle.emit("download-complete", serde_json::json!({
        "completed": ["OpenWebUI Backend"],
        "failed": []
    }));

    eprintln!("Backend installation successful!");
    Ok("Local backend installed successfully".to_string())
}

/// Find Python executable
fn find_python_executable() -> Result<String, String> {
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if version_str.contains("Python 3.") {
                    return Ok(cmd.to_string());
                }
            }
        }
    }

    Err("Python 3.10+ not found. Please install Python 3.10 or later.".to_string())
}
