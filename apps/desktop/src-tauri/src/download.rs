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

/// 获取下载目录
fn get_download_dir() -> Result<PathBuf, String> {
    let dirs = directories::UserDirs::new()
        .ok_or("Failed to get user directories".to_string())?;

    let download_dir = dirs.download_dir()
        .ok_or("Failed to get download directory".to_string())?;

    Ok(download_dir.join("open-webui"))
}

/// 构建下载 URL（支持镜像源）
fn build_download_url(base_url: &str, mirror_url: Option<&str>) -> String {
    if let Some(mirror) = mirror_url {
        // 如果有镜像源，尝试替换域名
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

/// 格式化下载速度
fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec < 1024 {
        format!("{} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024 * 1024 {
        format!("{:.2} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{:.2} MB/s", bytes_per_sec as f64 / (1024.0 * 1024.0))
    }
}

/// 下载单个文件
async fn download_file(
    url: &str,
    destination: &PathBuf,
    proxy_url: Option<&str>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // 创建 HTTP 客户端
    let client_builder = reqwest::Client::builder();

    let client = if let Some(proxy) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|e| format!("Invalid proxy URL: {}", e))?;
        client_builder.proxy(proxy).build()
    } else {
        client_builder.build()
    }.map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 发起请求
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

        // 计算下载速度
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        // 发送进度更新
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

    // 创建下载目录
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

        // 发送状态更新
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "downloading",
            "file": component.name,
            "url": download_url
        }));

        // 下载文件
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

    // 发送完成状态
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
    // TODO: 实现下载取消功能
    Ok("Download cancelled".to_string())
}

#[tauri::command]
pub fn get_download_dir_path() -> Result<String, String> {
    let path = get_download_dir()?;
    Ok(path.to_string_lossy().to_string())
}

/// 本地后端安装配置
#[derive(Debug, Serialize, Deserialize)]
pub struct BackendInstallConfig {
    pub pypi_mirror: Option<String>,
    pub proxy_url: Option<String>,
}

/// 安装本地后端（使用 pip 安装本地源码）
#[tauri::command]
pub async fn install_local_backend(
    config: Option<BackendInstallConfig>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    eprintln!("开始安装本地后端...");

    // 首先检查应用内是否已安装 Python
    let python_cmd = if let Ok(Some(python_path)) = get_installed_python() {
        eprintln!("使用应用内已安装的 Python: {:?}", python_path);
        python_path
    } else {
        // 应用内没有，检查系统 Python
        match find_python_executable() {
            Ok(cmd) => {
                eprintln!("使用系统 Python: {}", cmd);
                cmd
            }
            Err(_) => {
                // 系统也没有，下载并安装 Python
                eprintln!("系统未安装 Python，开始下载并安装...");

                // 发送 Python 安装开始事件
                let _ = app_handle.emit("download-status", serde_json::json!({
                    "status": "下载并安装 Python 环境...",
                    "file": "Python 3.11",
                    "url": "internal://python"
                }));

                // 安装 Python
                let install_result = install_python(app_handle.clone()).await
                    .map_err(|e| format!("Failed to install Python: {}", e))?;

                eprintln!("Python 安装成功: {:?}", install_result.python_path);
                install_result.python_path
            }
        }
    };

    eprintln!("使用 Python: {}", python_cmd);

    // 检查 pip 是否可用
    if !check_pip_installed(&python_cmd) {
        eprintln!("pip 不可用，开始安装 pip...");

        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "安装 pip...",
            "file": "Python pip",
            "url": "https://bootstrap.pypa.io/get-pip.py"
        }));

        // 获取 Python 目录
        let python_dir = std::path::PathBuf::from(&python_cmd)
            .parent()
            .ok_or("Failed to get Python directory".to_string())?
            .to_path_buf();

        // 创建 pip 安装配置
        let pip_config = crate::python_installer::PipInstallConfig {
            pypi_mirror: config.as_ref().and_then(|c| c.pypi_mirror.clone()),
            proxy_url: config.as_ref().and_then(|c| c.proxy_url.clone()),
        };

        // 安装 pip
        crate::python_installer::install_pip(&python_dir, &python_cmd, &app_handle, Some(&pip_config)).await?;
    }

    // 获取后端目录
    // 当前目录是 apps/desktop/src-tauri，需要向上 4 级找到项目根目录的 backend
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    eprintln!("当前目录: {:?}", current_dir);

    // 向上查找 backend 目录（最多向上 5 级）
    let mut search_dir = current_dir.as_path();
    let backend_dir = loop {
        if search_dir.join("backend").exists() {
            break search_dir.join("backend");
        }

        // 向上一级
        match search_dir.parent() {
            Some(parent) => {
                search_dir = parent;
                // 防止无限循环
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

    eprintln!("后端目录: {:?}", backend_dir);

    if !backend_dir.exists() {
        let error = format!("Backend directory not found: {:?}\nCurrent dir: {:?}", backend_dir, current_dir);
        eprintln!("{}", error);
        return Err(error);
    }

    // 发送进度 10% - 开始安装
    let progress = DownloadProgress {
        current: 10,
        total: 100,
        file: "OpenWebUI 后端".to_string(),
        speed: "准备中...".to_string(),
        percentage: 10.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    // 发送状态更新
    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "检查 Python 环境...",
        "file": "OpenWebUI 后端",
        "url": "local://backend"
    }));

    // 注意：pip 可用性检查已在上方（276-300行）完成，此处无需重复检查
    eprintln!("开始安装后端，Python: {}", python_cmd);

    // 发送进度 30% - 开始安装
    let progress = DownloadProgress {
        current: 30,
        total: 100,
        file: "OpenWebUI 后端".to_string(),
        speed: "安装依赖中...".to_string(),
        percentage: 30.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "安装 Python 依赖包...",
        "file": "OpenWebUI 后端",
        "url": "local://backend"
    }));

    eprintln!("开始安装后端，目录: {:?}", backend_dir);

    // 根据配置确定 PyPI 镜像源列表
    let pypi_mirrors = if let Some(cfg) = &config {
        if let Some(mirror) = &cfg.pypi_mirror {
            eprintln!("使用配置的 PyPI 镜像源: {}", mirror);
            vec![mirror.as_str(), "https://pypi.org/simple"]
        } else {
            vec![
                "https://mirrors.aliyun.com/pypi/simple/",      // 阿里云
                "https://pypi.tuna.tsinghua.edu.cn/simple/",    // 清华大学
                "https://pypi.org/simple",                      // 官方源（备选）
            ]
        }
    } else {
        vec![
            "https://mirrors.aliyun.com/pypi/simple/",      // 阿里云
            "https://pypi.tuna.tsinghua.edu.cn/simple/",    // 清华大学
            "https://pypi.org/simple",                      // 官方源（备选）
        ]
    };

    let mut install_success = false;
    let mut last_error = String::new();

    for (idx, mirror) in pypi_mirrors.iter().enumerate() {
        eprintln!("尝试使用镜像源 {}/{} 安装后端", idx + 1, mirror);

        // 使用辅助函数获取 site-packages 路径
        let site_packages = get_site_packages_path(&python_cmd);
        if let Some(ref sp) = site_packages {
            eprintln!("设置 PYTHONPATH: {:?}", sp);
        } else {
            eprintln!("警告: 无法获取 site-packages 路径");
        }

        // 使用 pip 安装后端依赖
        // 后端使用 requirements.txt 而不是 setup.py/pyproject.toml
        let requirements_file = backend_dir.join("requirements.txt");

        if !requirements_file.exists() {
            last_error = format!("requirements.txt not found in backend directory: {:?}", backend_dir);
            eprintln!("{}", last_error);
            continue;
        }

        // 发送安装状态更新
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": format!("正在安装依赖包（第 {} 个镜像源）...", idx + 1),
            "file": "OpenWebUI 后端",
            "url": "local://backend"
        }));

        eprintln!("开始执行 pip install，使用镜像源: {}", mirror);

        let mut cmd = create_python_command_with_env(&python_cmd);
        // 添加参数让 pip 输出更详细，并禁用进度条
        cmd.args([
            "-m", "pip", "install", "-r", "requirements.txt",
            "--index-url", mirror,
            "--progress-bar=off",
            "--verbose"  // 添加 verbose 输出
        ])
           .current_dir(&backend_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // 注意：不设置 PYTHONPATH，让 Python 自己处理路径
        // 完整版 Python 不需要手动设置 PYTHONPATH

        // 使用 spawn 来实时读取输出
        match cmd.spawn() {
            Ok(mut child) => {
                let (tx, rx) = mpsc::sync_channel::<String>(100);

                eprintln!("pip 进程已启动，PID: {:?}", child.id());

                // 读取 stdout（pip 的主要输出）
                if let Some(stdout) = child.stdout.take() {
                    let reader = std::io::BufReader::new(stdout);
                    let tx_clone = tx.clone();

                    // 在单独的线程中读取输出，通过通道发送
                    std::thread::spawn(move || {
                        eprintln!("[pip-reader] 线程已启动，开始读取输出...");
                        for line in reader.lines() {
                            if let Ok(line_text) = line {
                                let _ = tx_clone.send(line_text);
                            }
                        }
                        eprintln!("[pip-reader] 输出读取完成");
                    });
                }

                // 同时读取 stderr
                if let Some(stderr) = child.stderr.take() {
                    let reader = std::io::BufReader::new(stderr);
                    let tx_err = tx.clone();
                    std::thread::spawn(move || {
                        eprintln!("[pip-error] 错误读取线程已启动...");
                        for line in reader.lines() {
                            if let Ok(line_text) = line {
                                let _ = tx_err.send(format!("ERROR: {}", line_text));
                            }
                        }
                    });
                }

                // 在主线程上处理输出并发送事件
                // 使用 recv_timeout 避免永久阻塞
                let mut line_count = 0;
                let mut install_phase = 0.0; // 0-30: 收集依赖, 30-90: 下载安装, 90-100: 完成

                loop {
                    match rx.recv_timeout(std::time::Duration::from_millis(500)) {
                        Ok(line_text) => {
                            line_count += 1;

                            eprintln!("[pip {}] Raw: {}", line_count, line_text);

                            // 解析 pip 输出，提取当前安装的包
                            let (status, phase_increment) = if line_text.contains("Collecting ") {
                                let pkg = line_text.replace("Collecting ", "").trim().to_string();
                                let pkg_name = pkg.split_whitespace().next().unwrap_or(&pkg).to_string();
                                // 收集阶段：0-30%
                                (format!("正在收集: {}", pkg_name), 2.0)
                            } else if line_text.contains("Downloading ") {
                                let pkg = line_text.replace("Downloading ", "").trim().to_string();
                                let pkg_name = pkg.split_whitespace().next().unwrap_or(&pkg).to_string();
                                // 下载阶段：30-70%
                                (format!("正在下载: {}", pkg_name), 3.0)
                            } else if line_text.contains("Installing collected packages") {
                                ("开始安装已下载的包...".to_string(), 1.0)
                            } else if line_text.contains("Successfully installed ") {
                                ("安装完成！".to_string(), 0.5)
                            } else if line_text.contains("Requirement already satisfied") {
                                // 已安装的包，跳过
                                (format!("已安装: {}", line_text.trim()), 0.0)
                            } else if line_text.starts_with("ERROR:") {
                                (format!("错误: {}", &line_text[6..]), 0.0)
                            } else {
                                // 其他输出，小幅度增长
                                (format!("正在处理... (第 {} 行)", line_count), 0.1)
                            };

                            // 更新进度百分比，最高到 95%
                            install_phase = ((install_phase + phase_increment) as f64).min(95.0);

                            // 立即发送进度更新（每次都发送，不再限制频率）
                            let event_payload = serde_json::json!({
                                "status": status.clone(),
                                "file": "OpenWebUI 后端",
                                "url": "local://backend"
                            });

                            eprintln!("[pip EMIT] {} -> {} (进度: {}%)", line_count, status, install_phase);

                            match app_handle.emit("download-status", &event_payload) {
                                Ok(_) => {},
                                Err(e) => eprintln!("Failed to emit event: {}", e),
                            }

                            // 同时发送 download-progress 事件用于更新进度条
                            let progress = DownloadProgress {
                                current: line_count,
                                total: 100,
                                file: "OpenWebUI 后端".to_string(),
                                speed: status.clone(),
                                percentage: install_phase,
                            };
                            let _ = app_handle.emit("download-progress", &progress);
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // 超时，检查进程是否还在运行
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    // 进程已结束
                                    eprintln!("pip 进程已结束，退出码: {:?}", status.code());
                                    break;
                                }
                                Ok(None) => {
                                    // 进程还在运行，继续等待
                                    continue;
                                }
                                Err(_) => {
                                    // 无法检查进程状态，继续等待
                                    continue;
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            // 通道已断开，所有发送者都已关闭
                            eprintln!("pip 输出通道已断开");
                            break;
                        }
                    }
                }

                // 等待进程完成
                match child.wait() {
                    Ok(status) => {
                        eprintln!("pip install exit code: {:?}", status.code());

                        if status.success() {
                            eprintln!("后端安装成功（使用镜像源 {}）", mirror);
                            install_success = true;

                            // 发送成功状态
                            let _ = app_handle.emit("download-status", serde_json::json!({
                                "status": "依赖安装完成！",
                                "file": "OpenWebUI 后端",
                                "url": "local://backend"
                            }));
                            break;
                        } else {
                            last_error = format!("pip install failed with mirror {} (exit code: {:?})", mirror, status.code());
                            eprintln!("安装失败: {}", last_error);

                            // 发送失败状态，尝试下一个镜像源
                            let _ = app_handle.emit("download-status", serde_json::json!({
                                "status": format!("安装失败（第 {} 个镜像源），尝试下一个...", idx + 1),
                                "file": "OpenWebUI 后端",
                                "url": "local://backend"
                            }));
                        }
                    }
                    Err(e) => {
                        last_error = format!("Failed to wait for pip install: {}", e);
                        eprintln!("等待失败: {}", e);

                        // 发送执行失败状态
                        let _ = app_handle.emit("download-status", serde_json::json!({
                            "status": format!("执行失败（第 {} 个镜像源），尝试下一个...", idx + 1),
                            "file": "OpenWebUI 后端",
                            "url": "local://backend"
                        }));
                    }
                }
            }
            Err(e) => {
                last_error = format!("Failed to execute pip install: {}", e);
                eprintln!("执行失败: {}", e);

                // 发送执行失败状态
                let _ = app_handle.emit("download-status", serde_json::json!({
                    "status": format!("执行失败（第 {} 个镜像源），尝试下一个...", idx + 1),
                    "file": "OpenWebUI 后端",
                    "url": "local://backend"
                }));
            }
        }
    }

    if !install_success {
        return Err(format!("所有镜像源均失败。最后错误: {}", last_error));
    }

    // 验证 backend 目录是否存在
    eprintln!("验证 backend 目录...");
    let open_webui_dir = backend_dir.join("open_webui");
    if open_webui_dir.exists() {
        eprintln!("✓ backend/open_webui 目录存在");
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "OpenWebUI 后端文件验证完成！",
            "file": "OpenWebUI 后端",
            "url": "local://backend"
        }));
    } else {
        eprintln!("✗ backend/open_webui 目录不存在");
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": "警告: OpenWebUI 后端文件未找到，但继续执行...",
            "file": "OpenWebUI 后端",
            "url": "local://backend"
        }));
    }

    // 发送进度 100% - 完成
    let progress = DownloadProgress {
        current: 100,
        total: 100,
        file: "OpenWebUI 后端".to_string(),
        speed: "完成".to_string(),
        percentage: 100.0,
    };
    let _ = app_handle.emit("download-progress", &progress);

    // 等待一小段时间，确保所有事件都已发送到前端
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 发送完成事件
    let _ = app_handle.emit("download-complete", serde_json::json!({
        "completed": ["OpenWebUI 后端"],
        "failed": []
    }));

    eprintln!("后端安装成功!");
    Ok("Local backend installed successfully".to_string())
}

/// 查找 Python 可执行文件
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
