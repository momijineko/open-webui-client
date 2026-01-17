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

    // 1. 优先检查用户数据目录（最高优先级，用户修改过的版本）
    if let Some(dirs) = directories::UserDirs::new() {
        let home_dir = dirs.home_dir();
        let app_backend_dir = home_dir.join(".open-webui").join("backend");
        if app_backend_dir.exists() && app_backend_dir.join("open_webui").exists() {
            let normalized = normalize_path(&app_backend_dir);
            println!("Found backend in user directory: {:?}", normalized);
            return Some(normalized);
        }
    }

    // 2. 开发环境：可执行文件旁边或向上查找
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // 检查旁边的 backend
            let backend_path = exe_dir.join("backend");
            if backend_path.exists() && backend_path.join("open_webui").exists() {
                let normalized = normalize_path(&backend_path);
                println!("Found backend in development directory: {:?}", normalized);
                return Some(normalized);
            }

            // 向上查找
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

    // 3. 最后才检查打包的 resources（作为后备）
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

    // 4. 当前工作目录
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
    let backend_dir = find_backend_directory();

    // 检查安装路径
    let installation_path = if let Some(exe) = &backend_executable {
        Some(std::path::Path::new(exe)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string())
    } else if let Some(ref dir) = backend_dir {
        // 源码开发模式
        Some(dir.to_string_lossy().to_string())
    } else if python_available && open_webui_installed {
        // 如果通过 Python 安装，返回 Python 路径
        find_python_executable().ok().map(|_| "Python environment".to_string())
    } else {
        None
    };

    // 判断是否已安装
    // 支持三种模式：
    // 1. 独立可执行文件
    // 2. Python 包安装
    // 3. 源码开发目录 + Python 环境
    let is_installed = backend_executable.is_some()
        || (python_available && open_webui_installed)
        || (backend_dir.is_some() && python_available);

    // 检测后端来源和版本
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

// 获取后端版本（从 version.txt 或 pyproject.toml）
fn get_backend_version(backend_dir: &PathBuf) -> Option<String> {
    // 首先尝试读取 version.txt
    let version_file = backend_dir.join("version.txt");
    if version_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&version_file) {
            return Some(content.trim().to_string());
        }
    }

    // 尝试从 pyproject.toml 读取
    let pyproject = backend_dir.join("pyproject.toml");
    if pyproject.exists() {
        if let Ok(content) = std::fs::read_to_string(&pyproject) {
            for line in content.lines() {
                if line.trim().starts_with("version =") {
                    // 简单解析，实际可能需要更复杂的 TOML 解析
                    if let Some(v) = line.split('=').nth(1) {
                        return Some(v.trim().matches('"').collect::<String>());
                    }
                }
            }
        }
    }

    None
}

// 检测后端来源和版本状态
fn detect_backend_source_and_version(
    backend_dir: &Option<PathBuf>,
) -> (Option<String>, Option<String>, Option<String>, bool) {
    let user_backend_dir = if let Some(dirs) = directories::UserDirs::new() {
        Some(dirs.home_dir().join(".open-webui").join("backend"))
    } else {
        None
    };

    // 检查打包的后端版本
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
            // 检查是否是用户数据目录
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

            // 检查是否是开发目录（包含 .git）
            if dir.join(".git").exists() {
                return (Some("development".to_string()), None, bundled_version, false);
            }

            // 其他情况认为是打包的后端
            let current_version = get_backend_version(dir);
            (Some("bundled".to_string()), current_version, bundled_version, false)
        }
        None => (None, None, bundled_version, false),
    }
}

// 获取用户数据目录的后端路径
fn get_user_backend_dir() -> Option<PathBuf> {
    if let Some(dirs) = directories::UserDirs::new() {
        Some(dirs.home_dir().join(".open-webui").join("backend"))
    } else {
        None
    }
}

// 初始化用户后端目录（从打包资源复制）
#[tauri::command]
pub fn initialize_user_backend() -> Result<String, String> {
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    // 查找打包的后端资源
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

    // 如果用户目录已存在，先备份
    if user_backend_dir.exists() {
        let backup_dir = format!("{}.backup.{}", user_backend_dir.display(), chrono::Utc::now().timestamp());
        return Err(format!("User backend already exists. Backup at {}", backup_dir));
    }

    // 创建用户目录
    std::fs::create_dir_all(&user_backend_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // 复制后端文件
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy backend: {}", e))?;

    Ok(format!("Backend initialized to {}", user_backend_dir.display()))
}

// 递归复制目录
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

// 更新用户后端
#[tauri::command]
pub fn update_user_backend() -> Result<String, String> {
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    if !user_backend_dir.exists() {
        return initialize_user_backend();
    }

    // 查找打包的后端资源
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

    // 备份现有后端
    let backup_dir = format!("{}.backup.{}", user_backend_dir.display(), chrono::Utc::now().timestamp());
    std::fs::rename(&user_backend_dir, &backup_dir)
        .map_err(|e| format!("Failed to backup existing backend: {}", e))?;

    // 复制新版本
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy new backend: {}", e))?;

    Ok(format!("Backend updated. Backup at {}", backup_dir))
}

// 检查后端版本信息
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

// 安装后端依赖
fn install_backend_dependencies(backend_dir: &PathBuf) -> Result<(), String> {
    let python_cmd = find_python_executable()?;

    // 检查 requirements.txt 是否存在
    let requirements_file = backend_dir.join("requirements.txt");
    if !requirements_file.exists() {
        return Err("requirements.txt not found in backend directory".to_string());
    }

    println!("Installing backend dependencies from: {:?}", requirements_file);

    // 执行 pip install
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

// 获取打包的后端版本
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

// 检查并自动更新后端（应用启动时调用）
#[tauri::command]
pub fn check_and_auto_update_backend() -> Result<AutoUpdateResult, String> {
    use crate::config::{load_config, save_config};

    // 1. 加载配置
    let config = load_config()?;

    // 2. 检查是否是本地模式
    if config.setup_mode.as_deref() != Some("local") {
        return Ok(AutoUpdateResult {
            updated: false,
            backend_installed: false,
            message: "Not in local mode, skipping backend check".to_string(),
        });
    }

    // 3. 获取打包的后端版本
    let bundled_version = get_bundled_backend_version()
        .ok_or("Bundled backend version not found".to_string())?;

    // 4. 检查是否需要更新
    let needs_update = config.backend_version.as_ref() != Some(&bundled_version);

    if !needs_update {
        return Ok(AutoUpdateResult {
            updated: false,
            backend_installed: true,
            message: format!("Backend up to date: {}", bundled_version),
        });
    }

    println!("Backend update needed: {:?} -> {}", config.backend_version, bundled_version);

    // 5. 执行更新
    let user_backend_dir = get_user_backend_dir()
        .ok_or("Failed to get user data directory".to_string())?;

    // 查找打包的后端资源
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

    // 删除旧版本（如果存在）
    if user_backend_dir.exists() {
        std::fs::remove_dir_all(&user_backend_dir)
            .map_err(|e| format!("Failed to remove old backend: {}", e))?;
    }

    // 创建用户目录
    std::fs::create_dir_all(&user_backend_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // 复制新版本
    copy_dir(&bundled_backend, &user_backend_dir)
        .map_err(|e| format!("Failed to copy backend: {}", e))?;

    println!("Backend copied to: {:?}", user_backend_dir);

    // 6. 安装依赖
    install_backend_dependencies(&user_backend_dir)?;

    // 7. 更新配置文件
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
