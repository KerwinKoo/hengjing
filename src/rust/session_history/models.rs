use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: SessionSource,
    pub user_input: Option<String>,
    pub ai_response: String,
    pub selected_options: Vec<String>,
    pub images: Vec<ImageAttachment>,
}

/// 会话来源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionSource {
    Send,
    Continue,
    Enhance,
}

/// 图片附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub data: String, // Base64编码
    pub media_type: String,
    pub filename: Option<String>,
}

/// 会话数据（用于保存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub source: SessionSource,
    pub user_input: Option<String>,
    pub ai_response: String,
    pub selected_options: Vec<String>,
    pub images: Vec<ImageAttachment>,
}

/// 会话元数据（不包含截图数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: SessionSource,
    pub user_input: Option<String>,
    pub ai_response: String,
    pub selected_options: Vec<String>,
    pub image_count: usize,
    #[serde(default)]
    pub image_media_types: Vec<String>, // 存储每个图片的媒体类型
}

/// 侧边栏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarState {
    pub is_expanded: bool,
    pub width: f64,
}

/// 会话更新数据（用于更新已存在的会话）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdateData {
    pub source: SessionSource,
    pub user_input: Option<String>,
    pub selected_options: Vec<String>,
    pub images: Vec<ImageAttachment>,
}
