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

// 查找可用端口
fn find_available_port(start_port: u16) -> Option<u16> {
    for port in start_port..65535 {
        if port_is_available(port) {
            return Some(port);
        }
    }
    None
}

// 检查端口是否可用（简单检查）
fn port_is_available(port: u16) -> bool {
    // 在实际实现中，这里应该检查端口是否被占用
    // 暂时返回 true，假设端口可用
    port >= 1024 && port < 65535
}

// 查找 backend 目录
fn find_backend_directory() -> Option<PathBuf> {
    // 辅助函数：规范化路径，避免 Windows 长路径前缀 \\?\
    fn normalize_path(path: &PathBuf) -> PathBuf {
        // 使用 std::fs::canonicalize 但去除 \\?\ 前缀
        std::fs::canonicalize(path)
            .ok()
            .and_then(|p| {
                let s = p.to_string_lossy().to_string();
                // 去除 \\?\ 前缀（Windows 长路径前缀）
                if s.starts_with("\\\\?\\") {
                    Some(PathBuf::from(s[4..].to_string()))
                } else {
                    Some(p)
                }
            })
            .unwrap_or_else(|| path.clone())
    }

    // 1. 优先检查应用数据目录中的 backend（打包后的安装位置）
    if let Some(dirs) = directories::UserDirs::new() {
        let home_dir = dirs.home_dir();
        let app_backend_dir = home_dir.join(".open-webui").join("backend");
        if app_backend_dir.exists() && app_backend_dir.join("open_webui").exists() {
            let normalized = normalize_path(&app_backend_dir);
            println!("Found backend in app data directory: {:?}", normalized);
            return Some(normalized);
        }
    }

    // 2. 检查可执行文件旁边的 backend 目录（开发环境）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let backend_path = exe_dir.join("backend");
            if backend_path.exists() && backend_path.join("open_webui").exists() {
                let normalized = normalize_path(&backend_path);
                println!("Found backend next to executable: {:?}", normalized);
                return Some(normalized);
            }

            // 3. 向上查找 backend 目录（开发环境，可能在不同层级）
            let mut search_dir = exe_dir;
            let max_levels = 6; // 最多向上查找 6 级（开发环境可能需要）

            for _ in 0..max_levels {
                if let Some(parent) = search_dir.parent() {
                    search_dir = parent;
                    let backend_path = search_dir.join("backend");
                    if backend_path.exists() && backend_path.join("open_webui").exists() {
                        let normalized = normalize_path(&backend_path);
                        println!("Found backend by searching up: {:?}", normalized);
                        return Some(normalized);
                    }
                }
            }
        }
    }

    // 4. 最后尝试当前工作目录
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

// 检查进程是否还在运行
fn is_process_running(pid: u32) -> bool {
    // 使用系统命令检查进程是否存在
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
    // 先检查是否已有后端在运行
    let mut backend_guard = state.0.lock().unwrap();

    if let Some(backend) = backend_guard.as_ref() {
        if is_process_running(backend.pid) {
            return Ok(format!("Backend already running on port {}", backend.port));
        }
        // 旧进程已停止，清理
        *backend_guard = None;
    }

    // 查找可用端口
    let port = find_available_port(8080)
        .ok_or("No available port found".to_string())?;

    // 构建后端命令
    // 这里假设 Python 后端已经在系统中安装
    // 在实际实现中，应该使用打包后的可执行文件路径
    let python_cmd = find_python_executable()?;

    println!("Starting backend with Python: {}", python_cmd);

    // 查找 backend 目录并添加到 PYTHONPATH
    let backend_dir = find_backend_directory();
    if let Some(ref dir) = backend_dir {
        println!("Found backend directory: {:?}", dir);
    }

    let mut cmd = Command::new(&python_cmd);

    // 设置 PYTHONPATH 环境变量
    if let Some(ref dir) = backend_dir {
        // 获取当前的 PYTHONPATH（如果存在）
        let pythonpath = std::env::var("PYTHONPATH").unwrap_or_default();
        let new_pythonpath = if pythonpath.is_empty() {
            dir.to_string_lossy().to_string()
        } else {
            format!("{};{}", dir.to_string_lossy(), pythonpath)
        };
        cmd.env("PYTHONPATH", &new_pythonpath);
        println!("Set PYTHONPATH: {}", new_pythonpath);

        // 应用配置：设置环境变量
        if let Some(ref cfg) = config {
            // 设置 Hugging Face 镜像
            if let Some(ref hf_endpoint) = cfg.hf_endpoint {
                cmd.env("HF_ENDPOINT", hf_endpoint);
                println!("Set HF_ENDPOINT: {}", hf_endpoint);
            }

            // 设置离线模式
            if let Some(offline) = cfg.offline_mode {
                if offline {
                    cmd.env("OFFLINE_MODE", "true");
                    cmd.env("HF_HUB_OFFLINE", "1");
                    println!("Set OFFLINE_MODE: true");
                }
            }

            // 设置 PyPI 镜像（通过 PIP_INDEX_URL）
            if let Some(ref pypi_mirror) = cfg.pypi_mirror {
                cmd.env("PIP_INDEX_URL", pypi_mirror);
                println!("Set PIP_INDEX_URL: {}", pypi_mirror);
            }
        }

        // 使用 uvicorn 启动 FastAPI 应用
        // 参考 start.sh 和 start_windows.bat 中的启动方式
        // 注意：桌面应用不需要 --forwarded-allow-ips，且 * 可能被 shell 扩展导致问题
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

    // 启动进程
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();

            // 获取 stdout 和 stderr 的读取器
            let stdout = child.stdout.take().expect("Failed to get stdout");
            let stderr = child.stderr.take().expect("Failed to get stderr");

            // 创建日志文件路径
            let log_path = PathBuf::from("backend.log");
            let log_path_clone = log_path.clone();

            // 启动线程读取 stdout
            let stdout_thread = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("[Backend STDOUT] {}", line);
                        // 可选：写入日志文件
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

            // 启动线程读取 stderr
            let log_path_clone2 = log_path.clone();
            let stderr_thread = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[Backend STDERR] {}", line);
                        // 可选：写入日志文件
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

            // 保存进程信息
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
        // 尝试优雅地终止进程
        if backend.child.try_wait().unwrap().is_some() {
            // 进程已经停止
        } else {
            // 发送终止信号
            #[cfg(unix)]
            {
                let _ = Command::new("kill")
                    .arg(backend.pid.to_string())
                    .spawn();
            }

            #[cfg(windows)]
            {
                // Windows 下使用 taskkill 强制终止
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
        // 首先尝试使用 Child handle 检查进程状态
        match backend.child.try_wait() {
            Ok(Some(exit_status)) => {
                // 进程已经退出
                println!("Backend process exited with status: {:?}", exit_status);
                *backend_guard = None;
                return Ok(BackendStatus {
                    is_running: false,
                    port: None,
                    pid: None,
                });
            }
            Ok(None) => {
                // 进程还在运行
                let port = backend.port;
                let pid = backend.pid;
                return Ok(BackendStatus {
                    is_running: true,
                    port: Some(port),
                    pid: Some(pid),
                });
            }
            Err(e) => {
                // try_wait 失败，尝试使用系统命令检查
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
        // 读取日志文件
        match std::fs::read_to_string(&backend.log_file) {
            Ok(logs) => Ok(logs),
            Err(_) => Ok("No logs available".to_string()),
        }
    } else {
        Ok("No backend running".to_string())
    }
}

// 查找 Python 可执行文件
fn find_python_executable() -> Result<String, String> {
    // 首先检查应用内安装的 Python
    if let Ok(Some(python_path)) = crate::python_installer::get_installed_python() {
        return Ok(python_path);
    }

    // 如果应用内没有，再尝试常见的 Python 命令
    let commands = ["python3", "python", "py"];

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                // 检查版本是否 >= 3.10
                let version_str = String::from_utf8_lossy(&output.stdout);
                if version_str.contains("Python 3.") {
                    return Ok(cmd.to_string());
                }
            }
        }
    }

    Err("Python 3.10+ not found. Please install Python 3.10 or later.".to_string())
}

// 获取 Python 版本信息
fn get_python_version() -> Option<String> {
    // 首先检查应用内安装的 Python
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

    // 如果应用内没有，再尝试常见的 Python 命令
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

// 检查 OpenWebUI 是否已安装
fn check_open_webui_installed() -> bool {
    // 首先检查应用内安装的 Python
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

    // 如果应用内没有，再尝试常见的 Python 命令
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

// 检查后端可执行文件
fn check_backend_executable() -> Option<String> {
    // 检查应用数据目录中的后端可执行文件
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

    // 检查安装路径
    let installation_path = if let Some(exe) = &backend_executable {
        Some(std::path::Path::new(exe)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string())
    } else if python_available && open_webui_installed {
        // 如果通过 Python 安装，返回 Python 路径
        find_python_executable().ok().map(|_| "Python environment".to_string())
    } else {
        None
    };

    // 判断是否已安装
    let is_installed = backend_executable.is_some() || (python_available && open_webui_installed);

    Ok(BackendInstallationStatus {
        is_installed,
        installation_path,
        python_available,
        python_version,
        open_webui_installed,
        backend_executable,
    })
}
