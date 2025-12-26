use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;
use anyhow::Result;
use serde::Serialize;

/// CLI 工具安装目录
const INSTALL_DIR: &str = "/usr/local/bin";

/// CLI 安装状态
#[derive(Serialize)]
pub struct CliInstallStatus {
    pub installed: bool,
    pub hengjing_installed: bool,
    pub deng_installed: bool,
    pub install_dir: String,
    pub app_macos_dir: Option<String>,
    pub manual_commands: Option<String>,
}

/// 检查 CLI 工具是否已安装
pub fn is_cli_installed() -> bool {
    let hengjing_path = Path::new(INSTALL_DIR).join("恒境");
    let deng_path = Path::new(INSTALL_DIR).join("等");
    hengjing_path.exists() && deng_path.exists()
}

/// 获取当前应用的 MacOS 目录路径
fn get_app_macos_dir() -> Option<String> {
    if let Ok(exe_path) = std::env::current_exe() {
        // 检查是否在 .app 包内运行
        let path_str = exe_path.to_string_lossy();
        if path_str.contains(".app/Contents/MacOS") {
            if let Some(parent) = exe_path.parent() {
                return Some(parent.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// 安装 CLI 工具到系统路径
/// 
/// 创建符号链接从 /usr/local/bin 指向 .app 内的二进制文件
#[cfg(unix)]
pub fn install_cli_tools() -> Result<String> {
    let macos_dir = get_app_macos_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取应用路径，请确保从 .app 包运行"))?;
    
    let hengjing_src = format!("{}/恒境", macos_dir);
    let deng_src = format!("{}/等", macos_dir);
    
    // 检查源文件是否存在
    if !Path::new(&hengjing_src).exists() {
        return Err(anyhow::anyhow!("找不到恒境二进制文件: {}", hengjing_src));
    }
    if !Path::new(&deng_src).exists() {
        return Err(anyhow::anyhow!("找不到等二进制文件: {}", deng_src));
    }
    
    let hengjing_dst = format!("{}/恒境", INSTALL_DIR);
    let deng_dst = format!("{}/等", INSTALL_DIR);
    
    // 确保目标目录存在
    fs::create_dir_all(INSTALL_DIR)?;
    
    // 删除旧的符号链接（如果存在）
    let _ = fs::remove_file(&hengjing_dst);
    let _ = fs::remove_file(&deng_dst);
    
    // 创建新的符号链接
    symlink(&hengjing_src, &hengjing_dst)
        .map_err(|e| anyhow::anyhow!("创建恒境符号链接失败: {}。请尝试使用 sudo 运行或手动执行:\nsudo ln -sf {} {}", e, hengjing_src, hengjing_dst))?;
    
    symlink(&deng_src, &deng_dst)
        .map_err(|e| anyhow::anyhow!("创建等符号链接失败: {}。请尝试使用 sudo 运行或手动执行:\nsudo ln -sf {} {}", e, deng_src, deng_dst))?;
    
    Ok(format!("CLI 工具已安装到 {}", INSTALL_DIR))
}

#[cfg(not(unix))]
pub fn install_cli_tools() -> Result<String> {
    Err(anyhow::anyhow!("当前系统不支持自动安装，请手动配置 PATH"))
}

/// 获取手动安装命令
pub fn get_manual_install_commands() -> Option<String> {
    get_app_macos_dir().map(|macos_dir| {
        format!(
            "sudo ln -sf \"{}/恒境\" /usr/local/bin/恒境\nsudo ln -sf \"{}/等\" /usr/local/bin/等",
            macos_dir, macos_dir
        )
    })
}

// ============ Tauri 命令 ============

/// 获取 CLI 安装状态
#[tauri::command]
pub fn get_cli_install_status() -> CliInstallStatus {
    let hengjing_path = Path::new(INSTALL_DIR).join("恒境");
    let deng_path = Path::new(INSTALL_DIR).join("等");
    let app_macos_dir = get_app_macos_dir();
    
    CliInstallStatus {
        installed: hengjing_path.exists() && deng_path.exists(),
        hengjing_installed: hengjing_path.exists(),
        deng_installed: deng_path.exists(),
        install_dir: INSTALL_DIR.to_string(),
        app_macos_dir: app_macos_dir.clone(),
        manual_commands: get_manual_install_commands(),
    }
}

/// 安装 CLI 工具
#[tauri::command]
pub fn install_cli() -> Result<String, String> {
    install_cli_tools().map_err(|e| e.to_string())
}
