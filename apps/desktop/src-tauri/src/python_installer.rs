use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Python 安装进度
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PythonInstallProgress {
    pub current: u64,
    pub total: u64,
    pub file: String,
    pub speed: String,
    pub percentage: f64,
    pub stage: String, // "downloading", "extracting", "configuring"
}

/// Python 安装结果
#[derive(Debug, Serialize, Deserialize)]
pub struct PythonInstallResult {
    pub success: bool,
    pub python_path: String,
    pub version: String,
    pub message: String,
}

/// 获取 Python 数据目录
fn get_python_data_dir() -> Result<PathBuf, String> {
    let dirs = directories::UserDirs::new()
        .ok_or("Failed to get user directories".to_string())?;

    let home_dir = dirs.home_dir();
    let python_dir = home_dir.join(".open-webui").join("python");

    Ok(python_dir)
}

/// 获取平台特定的 Python 下载 URL 列表（使用完整 ZIP 包，自带 pip）
fn get_python_download_urls(version: &str) -> Vec<String> {
    let mut urls = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if cfg!(target_arch = "x86_64") {
            // 使用完整版 ZIP 包（非嵌入式版本）
            // 完整版格式: python-3.11.9-amd64.zip（自带 pip）
            // 嵌入式版本格式: python-3.11.9-embed-amd64.zip（无 pip）
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

/// 获取平台特定的 Python 下载 URL（保持向后兼容）
fn get_python_download_url(version: &str) -> String {
    get_python_download_urls(version).first().cloned().unwrap_or_default()
}

/// 下载文件并报告进度（带重试机制）
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
            eprintln!("重试下载 ({}/{}): {}", retry, max_retries, url);
            let _ = app_handle.emit("download-status", serde_json::json!({
                "status": format!("重试下载 ({}/{})...", retry, max_retries),
                "url": url
            }));
        }

        match download_file_once(url, destination, app_handle, stage_name, proxy_url).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("下载失败 (尝试 {}/{}): {}", retry + 1, max_retries, e);
                last_error = e;
            }
        }
    }

    Err(format!("下载失败，已重试 {} 次: {}", max_retries, last_error))
}

/// 单次下载尝试
async fn download_file_once(
    url: &str,
    destination: &PathBuf,
    app_handle: &AppHandle,
    stage_name: &str,
    proxy_url: Option<&str>,
) -> Result<(), String> {
    eprintln!("开始下载: {} -> {:?}", url, destination);

    // 创建 HTTP 客户端，支持代理和更长的超时时间
    let client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10分钟超时
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
            eprintln!("下载请求失败: {}", e);
            format!("Failed to fetch URL: {}", e)
        })?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // content_length 可能为 None
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

        // 计算下载速度
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };

        // 计算百分比（如果知道总大小）
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        // 发送进度更新
        let progress = PythonInstallProgress {
            current: downloaded,
            total: total_size.max(1), // 避免除以零
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

    eprintln!("下载完成: {:?}", destination);
    Ok(())
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

/// 解压 ZIP 文件
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

/// 安装 pip 配置
#[derive(Debug, Clone, Default)]
pub struct PipInstallConfig {
    pub pypi_mirror: Option<String>,
    pub proxy_url: Option<String>,
}

// get-pip.py 脚本内容（内置，避免网络下载）
// 修复版本：直接解压 wheel 到 site-packages，不依赖 pip 本身
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

/// 安装 pip 到嵌入式 Python
pub async fn install_pip(
    python_dir: &PathBuf,
    python_path: &str,
    _app_handle: &AppHandle,
    config: Option<&PipInstallConfig>,
) -> Result<(), String> {
    eprintln!("尝试使用 ensurepip 安装 pip...");

    // 首先尝试使用 ensurepip（如果可用）
    let ensurepip_output = std::process::Command::new(python_path)
        .args(["-m", "ensurepip", "--default-pip", "--upgrade"])
        .env("PYTHONPATH", python_dir.join("Lib").join("site-packages"))
        .output();

    if let Ok(output) = ensurepip_output {
        if output.status.success() {
            eprintln!("ensurepip 安装成功");
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("ensurepip stdout: {}", stdout);
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("ensurepip 失败: {}", stderr);
        }
    }

    // ensurepip 不可用，使用内置的 get-pip.py
    eprintln!("ensurepip 不可用，使用内置的 get-pip.py...");

    let get_pip_path = python_dir.join("get-pip.py");

    // 写入内置的 get-pip.py 脚本
    fs::write(&get_pip_path, GET_PIP_SCRIPT)
        .map_err(|e| format!("Failed to write get-pip.py: {}", e))?;

    eprintln!("get-pip.py 脚本已写入，尝试安装...");

    // 获取配置的 PyPI 镜像
    let pypi_mirror_url = config.and_then(|cfg| cfg.pypi_mirror.as_ref());

    let mut cmd = std::process::Command::new(python_path);
    cmd.arg(&get_pip_path)
        .env("PYTHONPATH", python_dir.join("Lib").join("site-packages"));

    // 如果配置了 PyPI 镜像，添加 index-url 参数
    if let Some(mirror) = &pypi_mirror_url {
        eprintln!("使用 PyPI 镜像源安装 pip: {}", mirror);
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
        eprintln!("pip 安装成功");

        // 清理 get-pip.py
        let _ = fs::remove_file(&get_pip_path);

        // 对于 Windows 嵌入式 Python，需要重新配置 .pth 文件
        #[cfg(target_os = "windows")]
        {
            eprintln!("重新配置 Python .pth 文件以启用 pip...");
            if let Err(e) = configure_embedded_python(python_dir) {
                eprintln!("警告: 配置 .pth 文件失败: {}", e);
            }

            // 额外措施：在 site-packages 中创建 pip.pth 文件
            // 这样即使主 .pth 文件不生效，pip 也能被找到
            let site_packages = python_dir.join("Lib").join("site-packages");
            let pip_pth = site_packages.join("pip.pth");

            // 创建一个指向 pip 包的 .pth 文件
            // 这会告诉 Python 将 pip 目录添加到 sys.path
            let pip_pth_content = format!("import site; site.addsitedir(r'{}')", site_packages.to_string_lossy().replace('\\', "/"));
            if let Err(e) = fs::write(&pip_pth, pip_pth_content) {
                eprintln!("警告: 创建 pip.pth 失败: {}", e);
            } else {
                eprintln!("已创建 pip.pth: {:?}", pip_pth);
            }

            // 调试：列出 site-packages 目录内容
            eprintln!("检查 site-packages 目录: {:?}", site_packages);
            if site_packages.exists() {
                if let Ok(entries) = fs::read_dir(&site_packages) {
                    for entry in entries.flatten() {
                        eprintln!("  - {:?}", entry.file_name());
                    }
                }
            } else {
                eprintln!("site-packages 目录不存在！");
            }

            // 调试：读取 .pth 文件内容
            if let Ok(entries) = fs::read_dir(python_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("pth") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            eprintln!(".pth 文件 {:?} 内容:\n{}", path, content);
                        }
                    }
                }
            }
        }

        // 验证 pip 是否真的可用（使用 PYTHONPATH）
        eprintln!("验证 pip 安装...");
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
                eprintln!("pip 验证 - stdout: {}, stderr: {}", stdout, stderr);
                if !output.status.success() {
                    eprintln!("警告: pip 模块无法导入");
                }
            }
            Err(e) => {
                eprintln!("pip 验证失败: {}", e);
            }
        }

        Ok(())
    } else {
        // 清理 get-pip.py
        let _ = fs::remove_file(&get_pip_path);
        Err(format!("get-pip.py failed:\nstdout: {}\nstderr: {}", stdout, stderr))
    }
}

/// 配置嵌入式 Python（Windows）
#[cfg(target_os = "windows")]
fn configure_embedded_python(python_dir: &PathBuf) -> Result<(), String> {
    eprintln!("configure_embedded_python called with: {:?}", python_dir);

    // 创建 site-packages 目录
    let site_packages = python_dir.join("Lib").join("site-packages");
    fs::create_dir_all(&site_packages)
        .map_err(|e| format!("Failed to create site-packages: {}", e))?;

    eprintln!("site-packages 目录: {:?}", site_packages);

    // 修改 python3xx._pth 文件以启用 site-packages
    let pth_files = fs::read_dir(python_dir)
        .map_err(|e| format!("Failed to read python dir: {}", e))?;

    let mut found_pth = false;
    for entry in pth_files {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let path = entry.path();

        eprintln!("检查文件: {:?}", path);

        if path.extension().and_then(|s| s.to_str()) == Some("pth") {
            found_pth = true;
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read pth file: {}", e))?;

            eprintln!("原始 .pth 文件内容:\n{}", content);

            let mut new_content = content.clone();

            // 取消注释 import site（这会启用 site.py 模块）
            if content.contains("# import site") {
                new_content = new_content.replace("# import site", "import site");
                eprintln!("已启用 'import site'");
            }

            // 使用绝对路径添加 site-packages
            // Windows .pth 文件需要正斜杠或双反斜杠
            let site_packages_str = site_packages.to_string_lossy().replace('\\', "/");

            if !content.contains("site-packages") && !content.contains(&site_packages_str) {
                new_content = format!("{}\n{}\n", new_content.trim(), site_packages_str);
                eprintln!("已添加 site-packages 路径: {}", site_packages_str);
            }

            eprintln!("新 .pth 文件内容:\n{}", new_content);

            fs::write(&path, new_content)
                .map_err(|e| format!("Failed to write pth file: {}", e))?;

            eprintln!("已配置 .pth 文件: {:?}", path);
        }
    }

    if !found_pth {
        eprintln!("警告: 未找到 .pth 文件!");
    }

    Ok(())
}

/// 检查 Python 是否已安装
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

/// 检查 pip 是否已安装
pub fn check_pip_installed(python_path: &str) -> bool {
    let mut cmd = create_python_command_with_env(python_path);
    cmd.args(["-m", "pip", "--version"]);

    if let Ok(output) = cmd.output() {
        output.status.success()
    } else {
        false
    }
}

/// 安装 Python（使用完整版 ZIP，自带 pip）
#[tauri::command]
pub async fn install_python(app_handle: AppHandle) -> Result<PythonInstallResult, String> {
    eprintln!("开始安装 Python...");

    let python_dir = get_python_data_dir()?;
    eprintln!("Python 数据目录: {:?}", python_dir);

    #[cfg(target_os = "windows")]
    let runtime_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let runtime_dir = python_dir.join("runtime");

    // 检查是否已安装
    if let Some(python_path) = check_python_installed(&runtime_dir) {
        eprintln!("Python 已安装: {:?}", python_path);

        // 完整版 Python 自带 pip，但验证一下
        if !check_pip_installed(&python_path) {
            eprintln!("警告: Python 已安装但 pip 不可用");
        }

        return Ok(PythonInstallResult {
            success: true,
            python_path,
            version: "3.11".to_string(),
            message: "Python already installed".to_string(),
        });
    }

    // 创建目录
    fs::create_dir_all(&python_dir)
        .map_err(|e| format!("Failed to create python directory: {}", e))?;

    let python_version = "3.11.9"; // 使用稳定的 Python 版本
    let download_urls = get_python_download_urls(python_version);

    eprintln!("下载 Python 从 {} 个镜像源", download_urls.len());

    // 下载路径
    let zip_filename = format!("python-{}.zip", python_version);
    let zip_path = python_dir.join(&zip_filename);

    // 尝试从多个镜像源下载
    let mut download_success = false;
    let mut last_error = String::new();

    for (idx, download_url) in download_urls.iter().enumerate() {
        eprintln!("尝试从镜像源 {}/{}: {}", idx + 1, download_urls.len(), download_url);

        // 发送开始下载事件
        let _ = app_handle.emit("download-status", serde_json::json!({
            "status": format!("下载 Python ({}/{})...", idx + 1, download_urls.len()),
            "url": download_url
        }));

        match download_file_with_progress(download_url, &zip_path, &app_handle, "downloading", None).await {
            Ok(()) => {
                eprintln!("下载成功: {}", download_url);
                download_success = true;
                break;
            }
            Err(e) => {
                eprintln!("从镜像源 {} 下载失败: {}", download_url, e);
                last_error = e;
                // 删除部分下载的文件
                let _ = fs::remove_file(&zip_path);
            }
        }
    }

    if !download_success {
        return Err(format!("所有镜像源均失败。最后错误: {}", last_error));
    }

    eprintln!("下载完成，开始解压...");

    // 发送解压事件
    let _ = app_handle.emit("download-status", serde_json::json!({
        "status": "解压中...",
        "url": download_urls.first().unwrap_or(&String::new())
    }));

    // 解压 Python
    #[cfg(target_os = "windows")]
    let extract_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let extract_dir = python_dir.join("runtime");

    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract directory: {}", e))?;

    extract_zip(&zip_path, &extract_dir)?;

    eprintln!("解压完成");

    // 清理下载的 zip 文件
    let _ = fs::remove_file(&zip_path);

    // 查找 Python 可执行文件
    // 完整版 ZIP 解压后可能有不同的目录结构
    let python_path = find_python_in_dir(&extract_dir)
        .ok_or("Python executable not found after installation".to_string())?;

    eprintln!("Python 安装成功: {:?}", python_path);

    // 验证 pip 是否可用（完整版应该自带）
    eprintln!("验证 pip 是否可用...");
    if check_pip_installed(&python_path) {
        eprintln!("pip 已就绪");
    } else {
        eprintln!("警告: pip 不可用，可能需要手动安装");
    }

    // 发送完成事件
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

/// 在目录中查找 Python 可执行文件
fn find_python_in_dir(dir: &PathBuf) -> Option<String> {
    // 首先尝试直接在目录中查找
    #[cfg(target_os = "windows")]
    let direct_exe = dir.join("python.exe");

    #[cfg(not(target_os = "windows"))]
    let direct_exe = dir.join("bin/python3");

    if direct_exe.exists() {
        return direct_exe.to_str().map(|s| s.to_string());
    }

    // 尝试在子目录中查找（完整版 ZIP 可能有嵌套目录）
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

/// 获取已安装的 Python 路径
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

/// 检查是否需要安装 Python
#[tauri::command]
pub fn check_python_needed() -> Result<bool, String> {
    // 首先检查系统 Python
    if find_system_python().is_some() {
        return Ok(false);
    }

    // 检查应用内 Python
    let python_dir = get_python_data_dir()?;

    #[cfg(target_os = "windows")]
    let runtime_dir = python_dir.join("runtime");

    #[cfg(not(target_os = "windows"))]
    let runtime_dir = python_dir.join("runtime");

    Ok(check_python_installed(&runtime_dir).is_none())
}

/// 查找系统 Python
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

/// 为嵌入式 Python 创建带有正确环境变量的命令
/// 完整版 Python 不需要特殊环境变量
pub fn create_python_command_with_env(python_path: &str) -> std::process::Command {
    std::process::Command::new(python_path)
}

/// 获取 Python 的 site-packages 路径
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
