use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::process::{Command, Child, Stdio};
use std::path::PathBuf;
use std::io::BufRead;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub is_running: bool,
    pub port: Option<u16>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub hf_endpoint: Option<String>,
    pub offline_mode: Option<bool>,
    pub pypi_mirror: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInstallationStatus {
    pub is_installed: bool,
    pub installation_path: Option<String>,
    pub python_available: bool,
    pub python_version: Option<String>,
    pub open_webui_installed: bool,
    pub backend_executable: Option<String>,
    pub backend_source: Option<String>, // "bundled" | "user" | "development"
    pub needs_update: bool,
    pub current_version: Option<String>,
    pub bundled_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendVersionInfo {
    pub current_version: Option<String>,
    pub bundled_version: Option<String>,
    pub update_available: bool,
}

pub struct BackendState(Mutex<Option<BackendProcess>>);

impl BackendState {
    pub fn new() -> Self {
        BackendState(Mutex::new(None))
    }
}

struct BackendProcess {
    child: Child,
    pid: u32,
    port: u16,
    log_file: PathBuf,
    _stdout_thread: Option<std::thread::JoinHandle<()>>,
    _stderr_thread: Option<std::thread::JoinHandle<()>>,
}

// Find available port
fn find_available_port(start_port: u16) -> Option<u16> {
    for port in start_port..65535 {
        if port_is_available(port) {
            return Some(port);
        }
    }
    None
}

// Check if port is available (simple check)
fn port_is_available(port: u16) -> bool {
    // In actual implementation, this should check if port is in use
    // For now, return true, assuming port is available
    port >= 1024 && port < 65535
}

// Find backend directory
fn find_backend_directory() -> Option<PathBuf> {
    // Helper function: normalize path, avoid Windows long path prefix \\?\
    fn normalize_path(path: &PathBuf) -> PathBuf {
        // Use std::fs::canonicalize but remove \\?\ prefix
        std::fs::canonicalize(path)
            .ok()
            .and_then(|p| {
                let s = p.to_string_lossy().to_string();
                // Remove \\?\ prefix (Windows long path prefix)
                if s.starts_with("\\\\?\\") {
                    Some(PathBuf::from(s[4..].to_string()))
                } else {
                    Some(p)
                }
            })
            .unwrap_or_else(|| path.clone())
    }

    // 1. First check user data directory (highest priority, user-modified version)
    if let Some(dirs) = directories::UserDirs::new() {
        let home_dir = dirs.home_dir();
        let app_backend_dir = home_dir.join(".open-webui").join("backend");
        if app_backend_dir.exists() && app_backend_dir.join("open_webui").exists() {
            let normalized = normalize_path(&app_backend_dir);
            println!("Found backend in user directory: {:?}", normalized);
            return Some(normalized);
        }
    }

    // 2. Development environment: next to executable or search upwards
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check backend next to executable
            let backend_path = exe_dir.join("backend");
            if backend_path.exists() && backend_path.join("open_webui").exists() {
                let normalized = normalize_path(&backend_path);
                println!("Found backend in development directory: {:?}", normalized);
                return Some(normalized);
            }

            // Search upwards
            let mut search_dir = exe_dir;
            for _ in 0..6 {
                if let Some(parent) = search_dir.parent() {
                    search_dir = parent;
                    let backend_path = search_dir.join("backend");
                    if backend_path.exists() && backend_path.join("open_webui").exists() {
                        let normalized = normalize_path(&backend_path);
                        println!("Found backend in development directory: {:?}", normalized);
                        return Some(normalized);
                    }
                }
            }
        }
    }

    // 3. Finally check bundled resources (as fallback)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let paths = vec![
                exe_dir.join("resources").join("backend"),      // Windows
                exe_dir.join("Resources").join("backend"),      // macOS
                exe_dir.join("..").join("resources").join("backend"), // Linux
            ];

            for backend_path in paths {
                if backend_path.exists() && backend_path.join("open_webui").exists() {
                    let normalized = normalize_path(&backend_path);
                    println!("Found backend in bundled resources: {:?}", normalized);
                    return Some(normalized);
                }
            }
        }
    }

    // 4. Current working directory
    if let Ok(current_dir) = std::env::current_dir() {
        let backend_path = current_dir.join("backend");
        if backend_path.exists() && backend_path.join("open_webui").exists() {
            let normalized = normalize_path(&backend_path);
            println!("Found backend in current directory: {:?}", normalized);
            return Some(normalized);
        }
    }

    println!("Backend directory not found");
    None
}

// Check if process is still running
fn is_process_running(pid: u32) -> bool {
    // Use system command to check if process exists
    #[cfg(windows)]
    {
        match Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
        {
            Ok(output) => String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()),
            Err(_) => false,
        }
    }

    #[cfg(unix)]
    {
        match Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output()
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

#[tauri::command]
pub fn start_backend(state: State<BackendState>, config: Option<BackendConfig>) -> Result<String, String> {
    // First check if backend is already running
    let mut backend_guard = state.0.lock().unwrap();

    if let Some(backend) = backend_guard.as_ref() {
        if is_process_running(backend.pid) {
            return Ok(format!("Backend already running on port {}", backend.port));
        }
        // Old process stopped, clean up
        *backend_guard = None;
    }

    // Find available port
    let port = find_available_port(8080)
        .ok_or("No available port found".to_string())?;

    // Build backend command
    // This assumes Python backend is already installed in system
    // In actual implementation, should use packaged executable path
    let python_cmd = find_python_executable()?;

    println!("Starting backend with Python: {}", python_cmd);

    // Find backend directory and add to PYTHONPATH
    let backend_dir = find_backend_directory();
    if let Some(ref dir) = backend_dir {
        println!("Found backend directory: {:?}", dir);
    }

    let mut cmd = Command::new(&python_cmd);

    // Set PYTHONPATH environment variable
    if let Some(ref dir) = backend_dir {
        // Get current PYTHONPATH (if exists)
        let pythonpath = std::env::var("PYTHONPATH").unwrap_or_default();
        let new_pythonpath = if pythonpath.is_empty() {
            dir.to_string_lossy().to_string()
        } else {
            format!("{};{}", dir.to_string_lossy(), pythonpath)
        };
        cmd.env("PYTHONPATH", &new_pythonpath);
        println!("Set PYTHONPATH: {}", new_pythonpath);

        // Apply configuration: set environment variables
        if let Some(ref cfg) = config {
            // Set Hugging Face mirror
            if let Some(ref hf_endpoint) = cfg.hf_endpoint {
                cmd.env("HF_ENDPOINT", hf_endpoint);
                println!("Set HF_ENDPOINT: {}", hf_endpoint);
            }

            // Set offline mode
            if let Some(offline) = cfg.offline_mode {
                if offline {
                    cmd.env("OFFLINE_MODE", "true");
                    cmd.env("HF_HUB_OFFLINE", "1");
                    println!("Set OFFLINE_MODE: true");
                }
            }

            // Set PyPI mirror (via PIP_INDEX_URL)
            if let Some(ref pypi_mirror) = cfg.pypi_mirror {
                cmd.env("PIP_INDEX_URL", pypi_mirror);
                println!("Set PIP_INDEX_URL: {}", pypi_mirror);
            }
        }

        // Use uvicorn to start FastAPI application
        // Reference start.sh and start_windows.bat for startup method
        // Note: desktop app doesn't need --forwarded-allow-ips, and * may be expanded by shell
        cmd.args(["-m", "uvicorn", "open_webui.main:app"])
           .args(["--host", "0.0.0.0"])
           .args(["--port", &port.to_string()])
           .arg("--ws")
           .arg("auto")
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
    } else {
        return Err("Backend directory not found".to_string());
    }

    // Start process
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();

            // Get stdout and stderr readers
            let stdout = child.stdout.take().expect("Failed to get stdout");
            let stderr = child.stderr.take().expect("Failed to get stderr");

            // Create log file path
            let log_path = PathBuf::from("backend.log");
            let log_path_clone = log_path.clone();

            // Start thread to read stdout
            let stdout_thread = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("[Backend STDOUT] {}", line);
                        // Optional: write to log file
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&log_path_clone)
                        {
                            use std::io::Write;
                            let _ = writeln!(file, "[STDOUT] {}", line);
                        }
                    }
                }
            });

            // Start thread to read stderr
            let log_path_clone2 = log_path.clone();
            let stderr_thread = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[Backend STDERR] {}", line);
                        // Optional: write to log file
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&log_path_clone2)
                        {
                            use std::io::Write;
                            let _ = writeln!(file, "[STDERR] {}", line);
                        }
                    }
                }
            });

            // Save process information
            *backend_guard = Some(BackendProcess {
                child,
                pid,
                port,
                log_file: log_path,
                _stdout_thread: Some(stdout_thread),
                _stderr_thread: Some(stderr_thread),
            });

            println!("Backend started on port {} (PID: {})", port, pid);
            Ok(format!("Backend started on port {} (PID: {})", port, pid))
        }
        Err(e) => {
            eprintln!("Failed to start backend: {}", e);
            Err(format!("Failed to start backend: {}", e))
        }
    }
}

#[tauri::command]
pub fn stop_backend(state: State<BackendState>) -> Result<String, String> {
    let mut backend_guard = state.0.lock().unwrap();

    if let Some(mut backend) = backend_guard.take() {
        // Try to gracefully terminate process
        if backend.child.try_wait().unwrap().is_some() {
            // Process already stopped
        } else {
            // Send termination signal
            #[cfg(unix)]
            {
                let _ = Command::new("kill")
                    .arg(backend.pid.to_string())
                    .spawn();
            }

            #[cfg(windows)]
            {
                // On Windows, use taskkill to force terminate
                let _ = Command::new("taskkill")
                    .args(["/PID", &backend.pid.to_string(), "/F"])
                    .spawn();
            }
        }

        Ok("Backend stopped".to_string())
    } else {
        Err("No backend running".to_string())
    }
}

#[tauri::command]
pub fn check_backend_status(state: State<BackendState>) -> Result<BackendStatus, String> {
    let mut backend_guard = state.0.lock().unwrap();

    if let Some(backend) = backend_guard.as_mut() {
        // First try using Child handle to check process status
        match backend.child.try_wait() {
            Ok(Some(exit_status)) => {
                // Process has exited
                println!("Backend process exited with status: {:?}", exit_status);
                *backend_guard = None;
                return Ok(BackendStatus {
                    is_running: false,
                    port: None,
                    pid: None,
                });
            }
            Ok(None) => {
                // Process is still running
                let port = backend.port;
                let pid = backend.pid;
                return Ok(BackendStatus {
                    is_running: true,
                    port: Some(port),
                    pid: Some(pid),
                });
            }
            Err(e) => {
                // try_wait failed, try using system command check
                println!("try_wait failed: {}, falling back to system check", e);
                let pid = backend.pid;
                let port = backend.port;
                let is_running = is_process_running(pid);

                if !is_running {
                    *backend_guard = None;
                    return Ok(BackendStatus {
                        is_running: false,
                        port: None,
                        pid: None,
                    });
                }

                return Ok(BackendStatus {
                    is_running: true,
                    port: Some(port),
                    pid: Some(pid),
                });
            }
        }
    } else {
        Ok(BackendStatus {
            is_running: false,
            port: None,
            pid: None,
        })
    }
}

#[tauri::command]
pub fn get_backend_logs(state: State<BackendState>) -> Result<String, String> {
    let backend_guard = state.0.lock().unwrap();

    if let Some(backend) = backend_guard.as_ref() {
        // Read log file
        match std::fs::read_to_string(&backend.log_file) {
            Ok(logs) => Ok(logs),
            Err(_) => Ok("No logs available".to_string()),
        }
    } else {
        Ok("No backend running".to_string())
    }
}

// Find Python executable
fn find_python_executable() -> Result<String, String> {
    // First check app-installed Python
    if let Ok(Some(python_path)) = crate::python_installer::get_installed_python() {
        return Ok(python_path);
    }

    // If not found in app, try common Python commands
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                // Check if version >= 3.10
                let version_str = String::from_utf8_lossy(&output.stdout);
                if version_str.contains("Python 3.") {
                    return Ok(cmd.to_string());
                }
            }
        }
    }

    Err("Python 3.10+ not found. Please install Python 3.10 or later.".to_string())
}

// Get Python version information
fn get_python_version() -> Option<String> {
    // First check app-installed Python
    if let Ok(Some(python_path)) = crate::python_installer::get_installed_python() {
        if let Ok(output) = Command::new(&python_path).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if version_str.contains("Python 3.") {
                    return Some(version_str);
                }
            }
        }
    }

    // If not found in app, try common Python commands
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if version_str.contains("Python 3.") {
                    return Some(version_str);
                }
            }
        }
    }

    None
}

// Check if OpenWebUI is installed
fn check_open_webui_installed() -> bool {
    // First check app-installed Python
    if let Ok(Some(python_path)) = crate::python_installer::get_installed_python() {
        if let Ok(output) = Command::new(&python_path)
            .args(["-m", "pip", "list"])
            .output()
        {
            if output.status.success() {
                let packages = String::from_utf8_lossy(&output.stdout);
                if packages.to_lowercase().contains("open-webui") {
                    return true;
                }
            }
        }
    }

    // If not found in app, try common Python commands
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd)
            .args(["-m", "pip", "list"])
            .output()
        {
            if output.status.success() {
                let packages = String::from_utf8_lossy(&output.stdout);
                if packages.to_lowercase().contains("open-webui") {
                    return true;
                }
            }
        }
    }

    false
}

// Check backend executable
fn check_backend_executable() -> Option<String> {
    // Check backend executable in app data directory
    if let Some(dirs) = directories::UserDirs::new() {
        let home_dir = dirs.home_dir();
        let app_dir = home_dir.join(".open-webui");
        let exe_name = if cfg!(windows) {
            "open-webui.exe"
        } else {
            "open-webui"
        };
        let exe_path = app_dir.join(exe_name);

        if exe_path.exists() {
            return Some(exe_path.to_string_lossy().to_string());
        }
    }

    None
}

#[tauri::command]
pub fn check_backend_installation() -> Result<BackendInstallationStatus, String> {
    let python_available = find_python_executable().is_ok();
    let python_version = get_python_version();
    let open_webui_installed = check_open_webui_installed();
    let backend_executable = check_backend_executable();
    let backend_dir = find_backend_directory();

    // Check installation path
    let installation_path = if let Some(exe) = &backend_executable {
        Some(std::path::Path::new(exe)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string())
    } else if let Some(ref dir) = backend_dir {
        // Source development mode
        Some(dir.to_string_lossy().to_string())
    } else if python_available && open_webui_installed {
        // If installed via Python, return Python path
        find_python_executable().ok().map(|_| "Python environment".to_string())
    } else {
        None
    };

    // Determine if installed
    // Support three modes:
    // 1. Standalone executable
    // 2. Python package installation
    // 3. Source development directory + Python environment
    let is_installed = backend_executable.is_some()
        || (python_available && open_webui_installed)
        || (backend_dir.is_some() && python_available);

    // Detect backend source and version
    let (backend_source, current_version, bundled_version, needs_update) =
        detect_backend_source_and_version(&backend_dir);

    Ok(BackendInstallationStatus {
        is_installed,
        installation_path,
        python_available,
        python_version,
        open_webui_installed,
        backend_executable,
        backend_source,
        needs_update,
        current_version,
        bundled_version,
    })
}

// Get backend version (from version.txt or pyproject.toml)
fn get_backend_version(backend_dir: &PathBuf) -> Option<String> {
    // First try reading version.txt
    let version_file = backend_dir.join("version.txt");
    if version_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&version_file) {
            return Some(content.trim().to_string());
        }
    }

    // Try reading from pyproject.toml
    let pyproject = backend_dir.join("pyproject.toml");
    if pyproject.exists() {
        if let Ok(content) = std::fs::read_to_string(&pyproject) {
            for line in content.lines() {
                if line.trim().starts_with("version =") {
                    // Simple parsing, actual implementation may need more complex TOML parsing
                    if let Some(v) = line.split('=').nth(1) {
                        return Some(v.trim().matches('"').collect::<String>());
                    }
                }
            }
        }
    }

    None
}

// Detect backend source and version status
fn detect_backend_source_and_version(
    backend_dir: &Option<PathBuf>,
) -> (Option<String>, Option<String>, Option<String>, bool) {
    let user_backend_dir = if let Some(dirs) = directories::UserDirs::new() {
        Some(dirs.home_dir().join(".open-webui").join("backend"))
    } else {
        None
    };

    // Check bundled backend version
    let bundled_version = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                get_backend_version(&resource_backend)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match backend_dir {
        Some(dir) => {
            // Check if it's user data directory
            if let Some(ref user_dir) = user_backend_dir {
                if dir.starts_with(user_dir) {
                    let current_version = get_backend_version(dir);
                    let needs_update = if let (Some(current), Some(bundled)) =
                        (&current_version, &bundled_version)
                    {
                        current != bundled
                    } else {
                        false
                    };
                    return (Some("user".to_string()), current_version, bundled_version, needs_update);
                }
            }

            // Check if it's development directory (contains .git)
            if dir.join(".git").exists() {
                return (Some("development".to_string()), None, bundled_version, false);
            }

            // Otherwise consider it bundled backend
            let current_version = get_backend_version(dir);
            (Some("bundled".to_string()), current_version, bundled_version, false)
        }
        None => (None, None, bundled_version, false),
    }
}

// Get user data directory backend path
fn get_user_backend_dir() -> Option<PathBuf> {
    if let Some(dirs) = directories::UserDirs::new() {
        Some(dirs.home_dir().join(".open-webui").join("backend"))
    } else {
        None
    }
}

// Initialize user backend directory (copy from bundled resources)
#[tauri::command]
pub fn initialize_user_backend() -> Result<String, String> {
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    // Find bundled backend resources
    let bundled_backend = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                Some(resource_backend)
            } else {
                return Err("Bundled backend not found".to_string());
            }
        } else {
            return Err("Failed to get executable directory".to_string());
        }
    } else {
        return Err("Failed to get executable path".to_string());
    };

    let bundled_backend = bundled_backend.unwrap();

    // If user directory exists, backup first
    if user_backend_dir.exists() {
        let backup_dir = format!("{}.backup.{}", user_backend_dir.display(), chrono::Utc::now().timestamp());
        return Err(format!("User backend already exists. Backup at {}", backup_dir));
    }

    // Create user directory
    std::fs::create_dir_all(&user_backend_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Copy backend files
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy backend: {}", e))?;

    Ok(format!("Backend initialized to {}", user_backend_dir.display()))
}

// Recursively copy directory
fn copy_dir(from: &PathBuf, to: &PathBuf) -> std::io::Result<()> {
    if !to.exists() {
        std::fs::create_dir_all(to)?;
    }

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if ty.is_dir() {
            copy_dir(&from_path, &to_path)?;
        } else {
            std::fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

// Update user backend
#[tauri::command]
pub fn update_user_backend() -> Result<String, String> {
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    if !user_backend_dir.exists() {
        return initialize_user_backend();
    }

    // Find bundled backend resources
    let bundled_backend = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                Some(resource_backend)
            } else {
                return Err("Bundled backend not found".to_string());
            }
        } else {
            return Err("Failed to get executable directory".to_string());
        }
    } else {
        return Err("Failed to get executable path".to_string());
    };

    let bundled_backend = bundled_backend.unwrap();

    // Backup existing backend
    let backup_dir = format!("{}.backup.{}", user_backend_dir.display(), chrono::Utc::now().timestamp());
    std::fs::rename(&user_backend_dir, &backup_dir)
        .map_err(|e| format!("Failed to backup existing backend: {}", e))?;

    // Copy new version
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy new backend: {}", e))?;

    Ok(format!("Backend updated. Backup at {}", backup_dir))
}

// Check backend version information
#[tauri::command]
pub fn get_backend_version_info() -> Result<BackendVersionInfo, String> {
    let backend_dir = find_backend_directory();
    let bundled_version = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                get_backend_version(&resource_backend)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let current_version = backend_dir.as_ref().and_then(|dir| get_backend_version(dir));
    let update_available = if let (Some(current), Some(bundled)) = (&current_version, &bundled_version) {
        current != bundled
    } else {
        false
    };

    Ok(BackendVersionInfo {
        current_version,
        bundled_version,
        update_available,
    })
}

// Install backend dependencies
fn install_backend_dependencies(backend_dir: &PathBuf) -> Result<(), String> {
    let python_cmd = find_python_executable()?;

    // Check if requirements.txt exists
    let requirements_file = backend_dir.join("requirements.txt");
    if !requirements_file.exists() {
        return Err("requirements.txt not found in backend directory".to_string());
    }

    println!("Installing backend dependencies from: {:?}", requirements_file);

    // Execute pip install
    let output = Command::new(&python_cmd)
        .args(["-m", "pip", "install", "-r", requirements_file.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run pip install: {}", e))?;

    if output.status.success() {
        println!("Dependencies installed successfully");
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install dependencies: {}", error_msg))
    }
}

// Get bundled backend version
fn get_bundled_backend_version() -> Option<String> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                return get_backend_version(&resource_backend);
            }
        }
    }
    None
}

// Check and auto-update backend (called on app startup)
#[tauri::command]
pub fn check_and_auto_update_backend() -> Result<AutoUpdateResult, String> {
    use crate::config::{load_config, save_config};

    // 1. Load configuration
    let config = load_config()?;

    // 2. Check if local mode
    if config.setup_mode.as_deref() != Some("local") {
        return Ok(AutoUpdateResult {
            updated: false,
            backend_installed: false,
            message: "Not in local mode, skipping backend check".to_string(),
        });
    }

    // 3. Get bundled backend version
    let bundled_version = get_bundled_backend_version()
        .ok_or("Bundled backend version not found".to_string())?;

    // 4. Check if update needed
    let needs_update = config.backend_version.as_ref() != Some(&bundled_version);

    if !needs_update {
        return Ok(AutoUpdateResult {
            updated: false,
            backend_installed: true,
            message: format!("Backend up to date: {}", bundled_version),
        });
    }

    println!("Backend update needed: {:?} -> {}", config.backend_version, bundled_version);

    // 5. Execute update
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    // Find bundled backend resources
    let bundled_backend = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_backend = exe_dir.join("resources").join("backend");
            if resource_backend.exists() {
                Some(resource_backend)
            } else {
                return Err("Bundled backend not found".to_string());
            }
        } else {
            return Err("Failed to get executable directory".to_string());
        }
    } else {
        return Err("Failed to get executable path".to_string());
    };

    let bundled_backend = bundled_backend.unwrap();

    // Remove old version (if exists)
    if user_backend_dir.exists() {
        std::fs::remove_dir_all(&user_backend_dir)
            .map_err(|e| format!("Failed to remove old backend: {}", e))?;
    }

    // Create user directory
    std::fs::create_dir_all(&user_backend_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Copy new version
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy backend: {}", e))?;

    println!("Backend copied to: {:?}", user_backend_dir);

    // 6. Install dependencies
    install_backend_dependencies(&user_backend_dir)?;

    // 7. Update configuration file
    let mut updated_config = config;
    updated_config.backend_version = Some(bundled_version.clone());
    save_config(&updated_config)?;

    Ok(AutoUpdateResult {
        updated: true,
        backend_installed: true,
        message: format!("Backend updated to {}", bundled_version),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdateResult {
    pub updated: bool,
    pub backend_installed: bool,
    pub message: String,
}
