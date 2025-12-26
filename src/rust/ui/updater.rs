use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf, process::Command};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProgress {
    pub chunk_length: usize,
    pub content_length: Option<u64>,
    pub downloaded: u64,
    pub percentage: f64,
}

/// 检查是否有可用更新
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    log::info!("🔍 开始检查更新");
    
    // 由于Tauri更新器无法处理中文tag，这里直接使用GitHub API检查
    let client = reqwest::Client::new();
    log::info!("📡 发送 GitHub API 请求");
    
    let response = client
        .get("https://api.github.com/repos/imhuso/continuum/releases/latest")
        .header("User-Agent", "continuum-app/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            log::error!("❌ 网络请求失败: {}", e);
            format!("网络请求失败: {}", e)
        })?;

    log::info!("📊 GitHub API 响应状态: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_msg = if status == 403 {
            "网络请求受限，请手动下载最新版本".to_string()
        } else if status == 404 {
            "网络连接异常，请检查网络后重试".to_string()
        } else {
            format!("网络请求失败: {}", status)
        };
        log::error!("❌ {}", error_msg);
        return Err(error_msg);
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            log::error!("❌ 解析响应失败: {}", e);
            format!("解析响应失败: {}", e)
        })?;

    log::info!("📋 成功获取 release 数据");

    let current_version = app.package_info().version.to_string();
    log::info!("📦 当前版本: {}", current_version);
    
    // 提取最新版本号，处理中文tag
    let tag_name = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    log::info!("🏷️ GitHub tag: {}", tag_name);
    
    // 移除前缀v和中文字符，只保留数字和点
    let latest_version = tag_name
        .replace("v", "")
        .chars()
        .filter(|c| c.is_numeric() || *c == '.')
        .collect::<String>();

    log::info!("🆕 解析后的最新版本: {}", latest_version);

    if latest_version.is_empty() {
        let error_msg = "无法解析版本号".to_string();
        log::error!("❌ {}", error_msg);
        return Err(error_msg);
    }

    // 比较版本号
    let has_update = compare_versions(&latest_version, &current_version);
    log::info!("🔄 版本比较结果 - 有更新: {}", has_update);

    // 获取实际的下载URL（从assets中找到对应平台的文件）
    let download_url = get_platform_download_url(&release)?;

    let update_info = UpdateInfo {
        available: has_update,
        current_version,
        latest_version,
        release_notes: release["body"].as_str().unwrap_or("").to_string(),
        download_url,
    };

    log::info!("✅ 更新检查完成: {:?}", update_info);
    Ok(update_info)
}

/// 简单的版本比较函数
fn compare_versions(v1: &str, v2: &str) -> bool {
    let v1_parts: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let v2_parts: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();
    
    let max_len = v1_parts.len().max(v2_parts.len());
    
    for i in 0..max_len {
        let v1_part = v1_parts.get(i).unwrap_or(&0);
        let v2_part = v2_parts.get(i).unwrap_or(&0);
        
        if v1_part > v2_part {
            return true;
        } else if v1_part < v2_part {
            return false;
        }
    }
    
    false
}

/// 下载并安装更新
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    log::info!("🚀 开始下载和安装更新");

    // 首先检查更新信息
    log::info!("🔍 重新检查更新信息");
    let update_info = check_for_updates(app.clone()).await?;

    log::info!("📊 更新信息: {:?}", update_info);

    if !update_info.available {
        let error_msg = "没有可用的更新".to_string();
        log::warn!("⚠️ {}", error_msg);
        return Err(error_msg);
    }

    log::info!("✅ 确认有可用更新，准备下载");

    // 发送下载开始事件
    log::info!("📢 发送下载开始事件");
    let _ = app.emit("update_download_started", ());

    // 实现真正的下载和安装逻辑
    match download_and_install_update_impl(&app, &update_info).await {
        Ok(_) => {
            log::info!("✅ 更新下载和安装成功");
            let _ = app.emit("update_install_finished", ());
            Ok(())
        }
        Err(e) => {
            log::error!("❌ 更新失败: {}", e);

            // 如果自动更新失败，提供手动下载选项
            log::info!("🔗 发送手动下载事件，URL: {}", update_info.download_url);
            let _ = app.emit("update_manual_download_required", &update_info.download_url);

            // 返回更友好的错误消息
            if e.contains("手动下载") {
                Err("请手动下载最新版本".to_string())
            } else {
                Err(format!("自动更新失败，请手动下载最新版本: {}", e))
            }
        }
    }
}

/// 获取当前应用版本
#[tauri::command]
pub async fn get_current_version(app: AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

/// 重启应用以完成更新
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}

/// 获取当前平台对应的下载URL
fn get_platform_download_url(release: &serde_json::Value) -> Result<String, String> {
    let assets = release["assets"].as_array()
        .ok_or_else(|| "无法获取release assets".to_string())?;

    log::info!("📦 Release assets 总数: {}", assets.len());

    // 确定当前平台（匹配实际的文件名格式）
    let platform = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "windows-aarch64"
        } else {
            "windows-x86_64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-aarch64"
        } else {
            "linux-x86_64"
        }
    } else {
        return Err("不支持的平台".to_string());
    };

    log::info!("🔍 查找平台 {} 的下载文件", platform);

    // 列出所有可用的 assets
    for (i, asset) in assets.iter().enumerate() {
        if let Some(name) = asset["name"].as_str() {
            log::info!("📄 Asset {}: {}", i + 1, name);
        }
    }

    // 查找对应平台的文件
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            log::info!("🔍 检查文件: {} (是否包含 '{}')", name, platform);
            if name.contains(platform) {
                if let Some(download_url) = asset["browser_download_url"].as_str() {
                    log::info!("✅ 找到匹配的下载文件: {}", name);
                    log::info!("🔗 下载URL: {}", download_url);
                    return Ok(download_url.to_string());
                }
            }
        }
    }

    // 如果找不到对应平台的文件，返回release页面URL作为fallback
    log::warn!("⚠️ 未找到平台 {} 的下载文件，使用release页面", platform);
    log::warn!("💡 可能的原因：1. 该平台没有预编译版本 2. 文件名格式不匹配");
    Ok(release["html_url"].as_str().unwrap_or("").to_string())
}

/// 实际的下载和安装实现
async fn download_and_install_update_impl(app: &AppHandle, update_info: &UpdateInfo) -> Result<(), String> {
    log::info!("🚀 开始自动更新实现");
    log::info!("📋 更新信息: {:?}", update_info);

    // 如果下载URL是GitHub页面而不是直接下载链接，引导用户手动下载
    if update_info.download_url.contains("/releases/tag/") {
        log::info!("🔗 下载URL是release页面，需要手动下载: {}", update_info.download_url);
        log::info!("💡 这通常意味着没有找到当前平台的预编译版本");
        return Err("请手动下载最新版本".to_string());
    }

    log::info!("📥 开始下载文件: {}", update_info.download_url);

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join("continuum_update");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    // 确定文件名
    let file_name = update_info.download_url
        .split('/')
        .last()
        .unwrap_or("update_file")
        .to_string();

    let file_path = temp_dir.join(&file_name);

    // 下载文件
    let client = reqwest::Client::new();
    let mut response = client
        .get(&update_info.download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let total_size = response.content_length();
    let mut downloaded = 0u64;
    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    // 下载并报告进度
    while let Some(chunk) = response.chunk().await
        .map_err(|e| format!("下载数据失败: {}", e))? {

        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        downloaded += chunk.len() as u64;

        let percentage = if let Some(total) = total_size {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let progress = UpdateProgress {
            chunk_length: chunk.len(),
            content_length: total_size,
            downloaded,
            percentage,
        };

        let _ = app.emit("update_download_progress", &progress);
    }

    log::info!("✅ 文件下载完成: {}", file_path.display());

    // 开始安装
    let _ = app.emit("update_install_started", ());

    // 根据平台执行不同的安装逻辑
    install_update(&file_path).await?;

    Ok(())
}

/// 根据平台安装更新
async fn install_update(file_path: &PathBuf) -> Result<(), String> {
    log::info!("🔧 开始安装更新: {}", file_path.display());

    if cfg!(target_os = "macos") {
        install_macos_update(file_path).await
    } else if cfg!(target_os = "windows") {
        install_windows_update(file_path).await
    } else if cfg!(target_os = "linux") {
        install_linux_update(file_path).await
    } else {
        Err("不支持的平台".to_string())
    }
}

/// macOS 安装逻辑
async fn install_macos_update(file_path: &PathBuf) -> Result<(), String> {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if file_name.ends_with(".tar.gz") {
        // 压缩包文件，需要解压并替换当前可执行文件
        log::info!("📦 处理 tar.gz 压缩包文件");
        install_from_archive(file_path).await
    } else if file_name.ends_with(".dmg") {
        // DMG 文件需要挂载后复制
        log::info!("📦 处理 DMG 文件");
        return Err("DMG 文件需要手动安装，请手动下载最新版本".to_string());
    } else {
        return Err("未知的文件格式，请手动下载最新版本".to_string());
    }
}

/// Windows 安装逻辑
async fn install_windows_update(file_path: &PathBuf) -> Result<(), String> {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if file_name.ends_with(".zip") {
        // ZIP 压缩包文件，需要解压并替换当前可执行文件
        log::info!("📦 处理 ZIP 压缩包文件");
        install_from_archive(file_path).await
    } else if file_name.ends_with(".msi") {
        // MSI 安装包
        log::info!("📦 执行 MSI 安装");
        let output = Command::new("msiexec")
            .args(&["/i", file_path.to_str().unwrap(), "/quiet"])
            .output()
            .map_err(|e| format!("执行 MSI 安装失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("MSI 安装失败: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    } else if file_name.ends_with(".exe") {
        // EXE 安装包
        log::info!("📦 执行 EXE 安装");
        let output = Command::new(file_path)
            .args(&["/S"]) // 静默安装
            .output()
            .map_err(|e| format!("执行 EXE 安装失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("EXE 安装失败: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    } else {
        Err("未知的文件格式，请手动下载最新版本".to_string())
    }
}

/// Linux 安装逻辑
async fn install_linux_update(file_path: &PathBuf) -> Result<(), String> {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if file_name.ends_with(".tar.gz") {
        // 压缩包文件，需要解压并替换当前可执行文件
        log::info!("📦 处理 tar.gz 压缩包文件");
        install_from_archive(file_path).await
    } else if file_name.ends_with(".deb") {
        // DEB 包
        log::info!("📦 执行 DEB 安装");
        let output = Command::new("dpkg")
            .args(&["-i", file_path.to_str().unwrap()])
            .output()
            .map_err(|e| format!("执行 DEB 安装失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("DEB 安装失败: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    } else if file_name.ends_with(".rpm") {
        // RPM 包
        log::info!("📦 执行 RPM 安装");
        let output = Command::new("rpm")
            .args(&["-U", file_path.to_str().unwrap()])
            .output()
            .map_err(|e| format!("执行 RPM 安装失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("RPM 安装失败: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    } else {
        Err("未知的文件格式，请手动下载最新版本".to_string())
    }
}

/// 从压缩包安装更新
async fn install_from_archive(file_path: &PathBuf) -> Result<(), String> {
    log::info!("📦 开始从压缩包安装更新: {}", file_path.display());

    // 获取当前可执行文件的路径
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;

    log::info!("📍 当前可执行文件路径: {}", current_exe.display());

    // 创建临时解压目录
    let temp_dir = std::env::temp_dir().join("continuum_extract");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("清理临时目录失败: {}", e))?;
    }
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时解压目录失败: {}", e))?;

    log::info!("📂 临时解压目录: {}", temp_dir.display());

    // 根据文件类型解压
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if file_name.ends_with(".tar.gz") {
        extract_tar_gz(file_path, &temp_dir)?;
    } else if file_name.ends_with(".zip") {
        extract_zip(file_path, &temp_dir)?;
    } else {
        return Err("不支持的压缩格式".to_string());
    }

    // 查找新的可执行文件
    let new_exe = find_executable_in_dir(&temp_dir)?;
    log::info!("🔍 找到新的可执行文件: {}", new_exe.display());

    // 替换当前可执行文件
    replace_executable(&current_exe, &new_exe)?;

    // 清理临时目录
    let _ = fs::remove_dir_all(&temp_dir);

    log::info!("✅ 更新安装完成！");
    Ok(())
}

/// 解压 tar.gz 文件
fn extract_tar_gz(archive_path: &PathBuf, extract_to: &PathBuf) -> Result<(), String> {
    log::info!("📦 解压 tar.gz 文件");

    let output = Command::new("tar")
        .args(&["-xzf", archive_path.to_str().unwrap(), "-C", extract_to.to_str().unwrap()])
        .output()
        .map_err(|e| format!("执行 tar 命令失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("tar 解压失败: {}", String::from_utf8_lossy(&output.stderr)));
    }

    log::info!("✅ tar.gz 解压完成");
    Ok(())
}

/// 解压 zip 文件
fn extract_zip(archive_path: &PathBuf, extract_to: &PathBuf) -> Result<(), String> {
    log::info!("📦 解压 zip 文件");

    // Windows 使用 PowerShell 解压
    if cfg!(target_os = "windows") {
        let ps_command = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            archive_path.display(),
            extract_to.display()
        );

        let output = Command::new("powershell")
            .args(&["-Command", &ps_command])
            .output()
            .map_err(|e| format!("执行 PowerShell 命令失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("PowerShell 解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
        // Unix 系统使用 unzip
        let output = Command::new("unzip")
            .args(&["-o", archive_path.to_str().unwrap(), "-d", extract_to.to_str().unwrap()])
            .output()
            .map_err(|e| format!("执行 unzip 命令失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("unzip 解压失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
    }

    log::info!("✅ zip 解压完成");
    Ok(())
}

/// 在目录中查找可执行文件
fn find_executable_in_dir(dir: &PathBuf) -> Result<PathBuf, String> {
    log::info!("🔍 在目录中查找可执行文件: {}", dir.display());

    // 递归查找目录中的所有文件
    fn find_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                find_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    find_files(dir, &mut files)?;

    log::info!("📋 解压后找到 {} 个文件", files.len());

    // 查找可执行文件
    for file in &files {
        if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
            log::info!("📄 检查文件: {} (路径: {})", file_name, file.display());

            // 查找名为 "等" 或 "continuum" 的可执行文件
            if file_name == "等" || file_name == "continuum" ||
               file_name == "等.exe" || file_name == "continuum.exe" ||
               file_name.starts_with("continuum") && !file_name.ends_with(".tar.gz") {
                log::info!("✅ 找到目标可执行文件: {}", file_name);
                return Ok(file.clone());
            }
        }
    }

    // 如果没找到明确的可执行文件，尝试查找任何可能的可执行文件
    log::warn!("⚠️ 未找到明确的可执行文件，尝试查找其他可能的文件");
    for file in &files {
        if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
            // 在 Unix 系统上，检查文件是否有执行权限
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(file) {
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 != 0 {
                        log::info!("🔍 找到有执行权限的文件: {}", file_name);
                        return Ok(file.clone());
                    }
                }
            }

            // 在 Windows 上，检查 .exe 文件
            #[cfg(windows)]
            {
                if file_name.ends_with(".exe") {
                    log::info!("🔍 找到 .exe 文件: {}", file_name);
                    return Ok(file.clone());
                }
            }
        }
    }

    Err(format!("在压缩包中未找到可执行文件。找到的文件: {:?}",
        files.iter().map(|f| f.file_name().and_then(|n| n.to_str()).unwrap_or("?")).collect::<Vec<_>>()))
}

/// 替换当前可执行文件
fn replace_executable(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<(), String> {
    log::info!("🔄 替换可执行文件");
    log::info!("📍 当前文件: {}", current_exe.display());
    log::info!("📍 新文件: {}", new_exe.display());

    // 创建备份（智能命名）
    let backup_path = create_backup_path(current_exe)?;

    log::info!("💾 创建当前文件备份: {}", backup_path.display());
    fs::copy(current_exe, &backup_path)
        .map_err(|e| format!("创建备份失败: {}", e))?;

    // 在 Windows 上，无法直接替换正在运行的可执行文件
    // 需要使用特殊的方法
    if cfg!(target_os = "windows") {
        replace_executable_windows(current_exe, new_exe)?;
    } else {
        replace_executable_unix(current_exe, new_exe)?;
    }

    log::info!("✅ 可执行文件替换完成");
    Ok(())
}

/// Windows 平台替换可执行文件
fn replace_executable_windows(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<(), String> {
    // Windows 上无法直接替换正在运行的文件
    // 创建一个批处理脚本来延迟替换
    let script_path = current_exe.parent().unwrap().join("update_script.bat");

    let script_content = format!(
        r#"@echo off
timeout /t 2 /nobreak >nul
copy /y "{}" "{}"
del "%~f0"
"#,
        new_exe.display(),
        current_exe.display()
    );

    fs::write(&script_path, script_content)
        .map_err(|e| format!("创建更新脚本失败: {}", e))?;

    log::info!("📝 创建 Windows 更新脚本: {}", script_path.display());
    log::info!("⚠️ Windows 平台需要重启应用以完成更新");

    // 启动脚本（不等待）
    Command::new("cmd")
        .args(&["/C", "start", "/min", script_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("启动更新脚本失败: {}", e))?;

    Ok(())
}

/// Unix 平台替换可执行文件
fn replace_executable_unix(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<(), String> {
    // 复制新文件到临时位置
    let temp_new = current_exe.with_extension("new");
    fs::copy(new_exe, &temp_new)
        .map_err(|e| format!("复制新文件失败: {}", e))?;

    // 设置执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_new)
            .map_err(|e| format!("获取文件权限失败: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_new, perms)
            .map_err(|e| format!("设置执行权限失败: {}", e))?;
    }

    // 原子性替换
    fs::rename(&temp_new, current_exe)
        .map_err(|e| format!("替换文件失败: {}", e))?;

    log::info!("✅ Unix 平台文件替换完成");
    Ok(())
}

/// 创建智能备份路径
fn create_backup_path(original_path: &PathBuf) -> Result<PathBuf, String> {
    let file_stem = original_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");

    let extension = original_path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let parent = original_path.parent()
        .ok_or_else(|| "无法获取文件父目录".to_string())?;

    // 获取当前版本信息，优先使用应用版本
    let current_version = get_current_app_version();

    // 基础备份文件名：xxx.version.bak
    let base_backup_name = if extension.is_empty() {
        format!("{}.{}.bak", file_stem, current_version)
    } else {
        format!("{}.{}.bak.{}", file_stem, current_version, extension)
    };

    let mut backup_path = parent.join(&base_backup_name);

    // 如果文件已存在，添加数字后缀
    let mut counter = 1;
    while backup_path.exists() {
        let numbered_backup_name = if extension.is_empty() {
            format!("{}.{}.bak.{}", file_stem, current_version, counter)
        } else {
            format!("{}.{}.bak.{}.{}", file_stem, current_version, counter, extension)
        };
        backup_path = parent.join(&numbered_backup_name);
        counter += 1;

        // 防止无限循环
        if counter > 100 {
            return Err("备份文件数量过多，请清理旧备份".to_string());
        }
    }

    log::info!("📝 生成备份文件路径: {}", backup_path.display());
    Ok(backup_path)
}

/// 获取当前应用版本
fn get_current_app_version() -> String {
    // 使用编译时嵌入的版本信息
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    // 验证版本格式
    if !VERSION.is_empty() && VERSION != "unknown" {
        log::info!("📋 使用编译时版本: {}", VERSION);
        return VERSION.to_string();
    }

    // 如果编译时版本不可用，尝试从应用名称中解析版本
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(file_name) = current_exe.file_name().and_then(|n| n.to_str()) {
            log::info!("🔍 尝试从文件名提取版本: {}", file_name);

            // 尝试匹配版本模式 (如 v1.2.3 或 1.2.3)
            if let Some(version) = extract_version_from_filename(file_name) {
                log::info!("✅ 从文件名提取到版本: {}", version);
                return version;
            }
        }
    }

    // 使用时间戳作为最后的fallback
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let fallback_version = format!("backup-{}", timestamp);
    log::warn!("⚠️ 无法获取版本信息，使用时间戳: {}", fallback_version);
    fallback_version
}

/// 从文件名中提取版本号
fn extract_version_from_filename(filename: &str) -> Option<String> {
    // 常见的版本模式
    let patterns = [
        r"v?(\d+\.\d+\.\d+)",  // v1.2.3 或 1.2.3
        r"v?(\d+\.\d+)",       // v1.2 或 1.2
        r"(\d+\.\d+\.\d+)",    // 纯数字版本
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.captures(filename) {
                if let Some(version) = captures.get(1) {
                    return Some(version.as_str().to_string());
                }
            }
        }
    }

    None
}
