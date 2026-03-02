use crate::config::AppState;
use crate::session_history::{SessionData, SessionRecord, SessionUpdateData, SidebarState, SessionStorageService};
use tauri::State;

/// 保存会话记录
#[tauri::command]
pub async fn save_session(
    session: SessionData,
    _state: State<'_, AppState>,
) -> Result<SessionRecord, String> {
    log::info!("[Command] save_session 被调用");

    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| {
            log::error!("[Command] 创建存储服务失败: {:?}", e);
            format!("Failed to create storage service: {}", e)
        })?;

    log::info!("[Command] 存储服务创建成功，开始保存会话");

    // 保存会话
    storage.save_session(session).await
        .map_err(|e| {
            log::error!("[Command] 保存会话失败: {:?}", e);

            // 返回用户友好的错误消息
            if e.to_string().contains("No space left") {
                "存储空间不足".to_string()
            } else if e.to_string().contains("Permission denied") {
                "文件权限不足".to_string()
            } else {
                format!("保存会话失败: {}", e)
            }
        })
        .map(|record| {
            log::info!("[Command] 会话保存成功: {}", record.id);
            record
        })
}

/// 更新会话记录
#[tauri::command]
pub async fn update_session(
    id: String,
    update: SessionUpdateData,
    _state: State<'_, AppState>,
) -> Result<SessionRecord, String> {
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;

    storage.update_session(&id, update).await
        .map_err(|e| {
            log::error!("Failed to update session {}: {:?}", id, e);
            format!("更新会话失败: {}", id)
        })
}

/// 加载所有会话记录（不包含截图数据）
#[tauri::command]
pub async fn load_sessions(
    _state: State<'_, AppState>,
) -> Result<Vec<SessionRecord>, String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 加载所有会话
    storage.load_sessions().await
        .map_err(|e| {
            log::error!("Failed to load sessions: {:?}", e);
            "加载会话列表失败".to_string()
        })
}

/// 获取单个会话（包含截图数据）
#[tauri::command]
pub async fn get_session(
    id: String,
    _state: State<'_, AppState>,
) -> Result<Option<SessionRecord>, String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 获取会话
    storage.get_session(&id).await
        .map_err(|e| {
            log::error!("Failed to get session {}: {:?}", id, e);
            format!("获取会话失败: {}", id)
        })
}

/// 删除单个会话
#[tauri::command]
pub async fn delete_session(
    id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 删除会话
    storage.delete_session(&id).await
        .map_err(|e| {
            log::error!("Failed to delete session {}: {:?}", id, e);
            format!("删除会话失败: {}", id)
        })
}

/// 批量删除会话
#[tauri::command]
pub async fn batch_delete_sessions(
    ids: Vec<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 批量删除会话
    storage.batch_delete_sessions(&ids).await
        .map_err(|e| {
            log::error!("Failed to batch delete sessions: {:?}", e);
            format!("批量删除会话失败: {}", e)
        })
}

/// 清空所有会话
#[tauri::command]
pub async fn clear_all_sessions(
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 清空所有会话
    storage.clear_all_sessions().await
        .map_err(|e| {
            log::error!("Failed to clear all sessions: {:?}", e);
            "清空所有会话失败".to_string()
        })
}

/// 保存侧边栏状态
#[tauri::command]
pub async fn save_sidebar_state(
    state_data: SidebarState,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 保存侧边栏状态
    storage.save_sidebar_state(&state_data).await
        .map_err(|e| {
            log::error!("Failed to save sidebar state: {:?}", e);
            "保存侧边栏状态失败".to_string()
        })
}

/// 加载侧边栏状态
#[tauri::command]
pub async fn load_sidebar_state(
    _state: State<'_, AppState>,
) -> Result<Option<SidebarState>, String> {
    // 创建存储服务实例
    let storage = SessionStorageService::new()
        .map_err(|e| format!("Failed to create storage service: {}", e))?;
    
    // 加载侧边栏状态
    storage.load_sidebar_state().await
        .map_err(|e| {
            log::error!("Failed to load sidebar state: {:?}", e);
            "加载侧边栏状态失败".to_string()
        })
}
