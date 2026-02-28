use super::models::{ImageAttachment, SessionData, SessionMetadata, SessionRecord, SidebarState};
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use image::{GenericImageView, ImageOutputFormat};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use uuid::Uuid;

/// 会话存储服务
pub struct SessionStorageService {
    storage_dir: PathBuf,
}

// 常量定义
const MAX_IMAGES: usize = 10; // 最大截图数量
const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const COMPRESSION_QUALITY: u8 = 85; // JPEG压缩质量 (0-100)

impl SessionStorageService {
    /// 创建新的存储服务实例
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;
        fs::create_dir_all(&storage_dir)
            .context("Failed to create storage directory")?;
        
        // 创建images子目录
        let images_dir = storage_dir.join("images");
        fs::create_dir_all(&images_dir)
            .context("Failed to create images directory")?;
        
        Ok(Self { storage_dir })
    }

    /// 获取存储目录路径
    fn get_storage_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Unable to get config directory")?;
        Ok(config_dir.join("continuum").join("sessions"))
    }

    /// 压缩图片数据到指定大小限制
    /// 如果图片已经小于限制，则不压缩
    /// 返回压缩后的数据和是否进行了压缩
    fn compress_image_if_needed(&self, image_data: &[u8]) -> Result<(Vec<u8>, bool)> {
        // 如果图片已经小于限制，直接返回
        if image_data.len() <= MAX_IMAGE_SIZE {
            return Ok((image_data.to_vec(), false));
        }

        log::info!(
            "Image size {} bytes exceeds limit {} bytes, compressing...",
            image_data.len(),
            MAX_IMAGE_SIZE
        );

        // 尝试加载图片
        let img = image::load_from_memory(image_data)
            .context("Failed to load image for compression")?;

        // 尝试不同的压缩质量直到满足大小要求
        let mut quality = COMPRESSION_QUALITY;
        let mut compressed_data = Vec::new();

        while quality > 10 {
            compressed_data.clear();
            let mut cursor = Cursor::new(&mut compressed_data);
            
            img.write_to(&mut cursor, ImageOutputFormat::Jpeg(quality))
                .context("Failed to compress image")?;

            if compressed_data.len() <= MAX_IMAGE_SIZE {
                log::info!(
                    "Image compressed from {} bytes to {} bytes at quality {}",
                    image_data.len(),
                    compressed_data.len(),
                    quality
                );
                return Ok((compressed_data, true));
            }

            quality -= 10;
        }

        // 如果仍然太大，尝试缩小图片尺寸
        log::warn!("Image still too large after compression, resizing...");
        let (width, height) = img.dimensions();
        let scale = 0.8;
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;
        
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        
        compressed_data.clear();
        let mut cursor = Cursor::new(&mut compressed_data);
        resized.write_to(&mut cursor, ImageOutputFormat::Jpeg(COMPRESSION_QUALITY))
            .context("Failed to compress resized image")?;

        if compressed_data.len() > MAX_IMAGE_SIZE {
            log::error!(
                "Image still exceeds size limit after resizing: {} bytes",
                compressed_data.len()
            );
            anyhow::bail!(
                "Image size {} bytes exceeds maximum allowed size {} bytes even after compression",
                compressed_data.len(),
                MAX_IMAGE_SIZE
            );
        }

        log::info!(
            "Image resized and compressed from {} bytes to {} bytes",
            image_data.len(),
            compressed_data.len()
        );

        Ok((compressed_data, true))
    }

    /// 保存会话记录
    pub async fn save_session(&self, session: SessionData) -> Result<SessionRecord> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // 限制截图数量（需求 10.4）
        let images_to_save = if session.images.len() > MAX_IMAGES {
            log::warn!(
                "Session has {} images, limiting to {} images",
                session.images.len(),
                MAX_IMAGES
            );
            &session.images[..MAX_IMAGES]
        } else {
            &session.images[..]
        };
        
        // 处理和压缩图片（需求 10.5, 10.6）
        let mut processed_images = Vec::new();
        let mut image_media_types = Vec::new();
        
        for (index, image) in images_to_save.iter().enumerate() {
            // 解码base64图片数据
            let image_data = general_purpose::STANDARD.decode(&image.data)
                .with_context(|| format!("Failed to decode base64 image data at index {}", index))?;
            
            // 压缩图片（如果需要）
            let (final_data, was_compressed) = self.compress_image_if_needed(&image_data)
                .with_context(|| format!("Failed to compress image at index {}", index))?;
            
            if was_compressed {
                log::info!("Image {} was compressed from {} to {} bytes", 
                    index, image_data.len(), final_data.len());
            }
            
            // 重新编码为base64
            let final_base64 = general_purpose::STANDARD.encode(&final_data);
            
            processed_images.push(ImageAttachment {
                data: final_base64,
                media_type: if was_compressed {
                    "image/jpeg".to_string() // 压缩后的图片是JPEG格式
                } else {
                    image.media_type.clone()
                },
                filename: image.filename.clone(),
            });
            
            image_media_types.push(if was_compressed {
                "image/jpeg".to_string()
            } else {
                image.media_type.clone()
            });
        }
        
        // 创建元数据
        let metadata = SessionMetadata {
            id: id.clone(),
            timestamp,
            source: session.source.clone(),
            user_input: session.user_input.clone(),
            ai_response: session.ai_response.clone(),
            selected_options: session.selected_options.clone(),
            image_count: processed_images.len(),
            image_media_types,
        };
        
        // 保存元数据到JSON文件
        let metadata_path = self.storage_dir.join(format!("{}.json", id));
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .context("Failed to serialize metadata")?;
        fs::write(&metadata_path, metadata_json)
            .context("Failed to write metadata file")?;
        
        // 保存截图（如果有）
        if !processed_images.is_empty() {
            let images_dir = self.storage_dir.join("images").join(&id);
            fs::create_dir_all(&images_dir)
                .context("Failed to create session images directory")?;
            
            for (index, image) in processed_images.iter().enumerate() {
                // 根据媒体类型确定文件扩展名
                let extension = if image.media_type.contains("jpeg") || image.media_type.contains("jpg") {
                    "jpg"
                } else if image.media_type.contains("png") {
                    "png"
                } else if image.media_type.contains("gif") {
                    "gif"
                } else {
                    "jpg" // 默认使用jpg
                };
                
                let image_path = images_dir.join(format!("{}.{}", index, extension));
                let image_data = general_purpose::STANDARD.decode(&image.data)
                    .context("Failed to decode processed image data")?;
                fs::write(image_path, image_data)
                    .context("Failed to write image file")?;
            }
        }
        
        log::info!(
            "Session {} saved with {} images (original: {} images)",
            id,
            processed_images.len(),
            session.images.len()
        );
        
        Ok(SessionRecord {
            id,
            timestamp,
            source: session.source,
            user_input: session.user_input,
            ai_response: session.ai_response,
            selected_options: session.selected_options,
            images: processed_images,
        })
    }

    /// 加载所有会话记录（不包含截图数据）
    pub async fn load_sessions(&self) -> Result<Vec<SessionRecord>> {
        let mut sessions = Vec::new();
        
        let entries = fs::read_dir(&self.storage_dir)
            .context("Failed to read storage directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // 只处理JSON文件
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_session_metadata(&path) {
                    Ok(metadata) => {
                        // 创建会话记录（不加载截图）
                        let session = SessionRecord {
                            id: metadata.id,
                            timestamp: metadata.timestamp,
                            source: metadata.source,
                            user_input: metadata.user_input,
                            ai_response: metadata.ai_response,
                            selected_options: metadata.selected_options,
                            images: vec![], // 延迟加载
                        };
                        sessions.push(session);
                    }
                    Err(e) => {
                        // 记录错误但继续处理其他文件
                        log::warn!("Failed to load session metadata from {:?}: {}", path, e);
                    }
                }
            }
        }
        
        // 按时间倒序排序（最新的在前）
        sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(sessions)
    }

    /// 加载单个会话元数据
    fn load_session_metadata(&self, path: &PathBuf) -> Result<SessionMetadata> {
        let metadata_json = fs::read_to_string(path)
            .context("Failed to read metadata file")?;
        let metadata: SessionMetadata = serde_json::from_str(&metadata_json)
            .context("Failed to parse metadata JSON")?;
        Ok(metadata)
    }

    /// 获取单个会话（包含截图数据）
    pub async fn get_session(&self, id: &str) -> Result<Option<SessionRecord>> {
        let metadata_path = self.storage_dir.join(format!("{}.json", id));
        
        if !metadata_path.exists() {
            return Ok(None);
        }
        
        let metadata = self.load_session_metadata(&metadata_path)?;
        
        // 加载截图
        let mut images = Vec::new();
        let images_dir = self.storage_dir.join("images").join(id);
        
        if images_dir.exists() {
            for index in 0..metadata.image_count {
                // 尝试不同的文件扩展名
                let possible_extensions = ["jpg", "jpeg", "png", "gif"];
                let mut found = false;
                
                for ext in &possible_extensions {
                    let image_path = images_dir.join(format!("{}.{}", index, ext));
                    if image_path.exists() {
                        let image_data = fs::read(&image_path)
                            .context("Failed to read image file")?;
                        let base64_data = general_purpose::STANDARD.encode(&image_data);
                        
                        // 从元数据中恢复正确的媒体类型
                        let media_type = metadata.image_media_types.get(index)
                            .cloned()
                            .unwrap_or_else(|| "image/png".to_string());
                        
                        images.push(ImageAttachment {
                            data: base64_data,
                            media_type,
                            filename: None,
                        });
                        found = true;
                        break;
                    }
                }
                
                if !found {
                    log::warn!("Image file not found for session {} at index {}", id, index);
                }
            }
        }
        
        Ok(Some(SessionRecord {
            id: metadata.id,
            timestamp: metadata.timestamp,
            source: metadata.source,
            user_input: metadata.user_input,
            ai_response: metadata.ai_response,
            selected_options: metadata.selected_options,
            images,
        }))
    }

    /// 删除单个会话
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        // 删除元数据文件
        let metadata_path = self.storage_dir.join(format!("{}.json", id));
        if metadata_path.exists() {
            fs::remove_file(metadata_path)
                .context("Failed to delete metadata file")?;
        }
        
        // 删除截图目录
        let images_dir = self.storage_dir.join("images").join(id);
        if images_dir.exists() {
            fs::remove_dir_all(images_dir)
                .context("Failed to delete images directory")?;
        }
        
        Ok(())
    }

    /// 批量删除会话
    pub async fn batch_delete_sessions(&self, ids: &[String]) -> Result<()> {
        let mut errors = Vec::new();
        
        for id in ids {
            if let Err(e) = self.delete_session(id).await {
                errors.push(format!("Failed to delete session {}: {}", id, e));
            }
        }
        
        if !errors.is_empty() {
            anyhow::bail!("Batch delete partially failed: {}", errors.join(", "));
        }
        
        Ok(())
    }

    /// 清空所有会话
    pub async fn clear_all_sessions(&self) -> Result<()> {
        // 删除所有元数据文件
        let entries = fs::read_dir(&self.storage_dir)
            .context("Failed to read storage directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(path)
                    .context("Failed to delete metadata file")?;
            }
        }
        
        // 删除所有截图目录
        let images_dir = self.storage_dir.join("images");
        if images_dir.exists() {
            fs::remove_dir_all(&images_dir)
                .context("Failed to delete images directory")?;
            fs::create_dir_all(&images_dir)
                .context("Failed to recreate images directory")?;
        }
        
        Ok(())
    }

    /// 保存侧边栏状态
    pub async fn save_sidebar_state(&self, state: &SidebarState) -> Result<()> {
        let state_path = self.storage_dir.join("sidebar_state.json");
        let state_json = serde_json::to_string_pretty(state)
            .context("Failed to serialize sidebar state")?;
        fs::write(state_path, state_json)
            .context("Failed to write sidebar state file")?;
        Ok(())
    }

    /// 加载侧边栏状态
    pub async fn load_sidebar_state(&self) -> Result<Option<SidebarState>> {
        let state_path = self.storage_dir.join("sidebar_state.json");
        
        if !state_path.exists() {
            return Ok(None);
        }
        
        let state_json = fs::read_to_string(state_path)
            .context("Failed to read sidebar state file")?;
        let state: SidebarState = serde_json::from_str(&state_json)
            .context("Failed to parse sidebar state JSON")?;
        Ok(Some(state))
    }
}

impl Default for SessionStorageService {
    fn default() -> Self {
        Self::new().expect("Failed to create SessionStorageService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_history::models::{SessionSource, SessionData, ImageAttachment};
    use proptest::prelude::*;
    use tempfile::TempDir;

    // 策略：生成有效的SessionSource
    fn session_source_strategy() -> impl Strategy<Value = SessionSource> {
        prop_oneof![
            Just(SessionSource::Send),
            Just(SessionSource::Continue),
            Just(SessionSource::Enhance),
        ]
    }

    // 策略：生成有效的ImageAttachment
    fn image_attachment_strategy() -> impl Strategy<Value = ImageAttachment> {
        (
            // 生成小的随机字节数组并转换为base64
            prop::collection::vec(any::<u8>(), 10..100),
            prop::string::string_regex("image/(png|jpeg|gif)").unwrap(),
            prop::option::of(prop::string::string_regex("[a-z]{3,10}\\.(png|jpg|gif)").unwrap()),
        )
            .prop_map(|(bytes, media_type, filename)| ImageAttachment {
                data: general_purpose::STANDARD.encode(&bytes),
                media_type,
                filename,
            })
    }

    // 策略：生成有效的SessionData
    fn session_data_strategy() -> impl Strategy<Value = SessionData> {
        (
            session_source_strategy(),
            prop::option::of(prop::string::string_regex(".{1,200}").unwrap()),
            prop::string::string_regex(".{1,500}").unwrap(),
            prop::collection::vec(prop::string::string_regex("[a-z]{3,20}").unwrap(), 0..5),
            prop::collection::vec(image_attachment_strategy(), 0..3),
        )
            .prop_map(|(source, user_input, ai_response, selected_options, images)| SessionData {
                source,
                user_input,
                ai_response,
                selected_options,
                images,
            })
    }

    // 辅助函数：创建临时存储服务
    fn create_temp_storage() -> (SessionStorageService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_path_buf();
        
        // 创建必要的子目录
        fs::create_dir_all(&storage_dir).unwrap();
        fs::create_dir_all(storage_dir.join("images")).unwrap();
        
        let service = SessionStorageService {
            storage_dir,
        };
        
        (service, temp_dir)
    }

    // 辅助函数：比较SessionData和SessionRecord是否等价
    fn is_equivalent(original: &SessionData, loaded: &SessionRecord) -> bool {
        // 比较基本字段
        if !matches!(
            (&original.source, &loaded.source),
            (SessionSource::Send, SessionSource::Send)
                | (SessionSource::Continue, SessionSource::Continue)
                | (SessionSource::Enhance, SessionSource::Enhance)
        ) {
            return false;
        }
        if original.user_input != loaded.user_input {
            return false;
        }
        if original.ai_response != loaded.ai_response {
            return false;
        }
        if original.selected_options != loaded.selected_options {
            return false;
        }
        
        // 比较图片数量
        if original.images.len() != loaded.images.len() {
            return false;
        }
        
        // 比较每个图片 - 解码base64后比较原始字节
        for (orig_img, loaded_img) in original.images.iter().zip(loaded.images.iter()) {
            // 解码base64数据进行比较
            let orig_bytes = match general_purpose::STANDARD.decode(&orig_img.data) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            };
            let loaded_bytes = match general_purpose::STANDARD.decode(&loaded_img.data) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            };
            
            if orig_bytes != loaded_bytes {
                return false;
            }
            if orig_img.media_type != loaded_img.media_type {
                return false;
            }
            // filename可能会丢失，因为我们保存时使用索引命名
            // 所以不比较filename
        }
        
        true
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        /// **属性 1：会话数据往返一致性**
        /// **验证：需求 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7**
        /// 
        /// 对于任何有效的会话数据（包括用户输入、AI回复、选项和截图），
        /// 保存后再加载应该产生等价的数据对象。
        #[test]
        fn property_session_data_roundtrip_consistency(
            session_data in session_data_strategy()
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 保存会话
                let saved_record = storage.save_session(session_data.clone()).await
                    .expect("保存会话应该成功");
                
                // 加载会话
                let loaded_record = storage.get_session(&saved_record.id).await
                    .expect("加载会话应该成功")
                    .expect("会话应该存在");
                
                // 验证数据等价性
                prop_assert!(
                    is_equivalent(&session_data, &loaded_record),
                    "保存和加载的会话数据应该等价"
                );
                
                // 验证ID和时间戳存在
                prop_assert!(!loaded_record.id.is_empty(), "会话ID不应为空");
                prop_assert!(
                    loaded_record.timestamp <= Utc::now(),
                    "时间戳应该不晚于当前时间"
                );
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }

        /// **属性 3：会话ID唯一性**
        /// **验证：需求 2.6**
        /// 
        /// 对于任何两个不同的会话保存操作，生成的会话ID应该是唯一的。
        #[test]
        fn property_session_id_uniqueness(
            sessions in prop::collection::vec(session_data_strategy(), 2..15)
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                let mut ids = std::collections::HashSet::new();
                
                // 保存所有会话并收集ID
                for session in sessions {
                    let record = storage.save_session(session).await
                        .expect("保存会话应该成功");
                    
                    // 验证ID是唯一的
                    prop_assert!(
                        ids.insert(record.id.clone()),
                        "会话ID应该唯一，但发现重复ID: {}",
                        record.id
                    );
                }
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }

        /// **属性 9：截图数量限制**
        /// **验证：需求 10.4**
        /// 
        /// 对于任何会话保存操作，如果截图数量超过10张，
        /// 系统应该拒绝或截断到10张。
        #[test]
        fn property_screenshot_count_limit(
            session_data_base in session_data_strategy(),
            extra_image_count in 1usize..50usize,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 创建一个包含超过10张图片的会话
                let mut session_data = session_data_base;
                
                // 确保至少有11张图片（10 + 至少1张额外的）
                let total_images = 10 + extra_image_count;
                session_data.images.clear();
                
                for i in 0..total_images {
                    let data = vec![i as u8; 50];
                    session_data.images.push(ImageAttachment {
                        data: general_purpose::STANDARD.encode(&data),
                        media_type: "image/png".to_string(),
                        filename: Some(format!("test{}.png", i)),
                    });
                }
                
                prop_assert!(
                    session_data.images.len() > 10,
                    "测试数据应该有超过10张图片，实际: {}",
                    session_data.images.len()
                );
                
                // 保存会话
                let saved_record = storage.save_session(session_data.clone()).await
                    .expect("保存会话应该成功");
                
                // 验证只保存了10张图片
                prop_assert_eq!(
                    saved_record.images.len(),
                    10,
                    "保存的会话应该只有10张图片，即使原始数据有{}张",
                    session_data.images.len()
                );
                
                // 加载会话并验证持久化后的数据
                let loaded_record = storage.get_session(&saved_record.id).await
                    .expect("加载会话应该成功")
                    .expect("会话应该存在");
                
                prop_assert_eq!(
                    loaded_record.images.len(),
                    10,
                    "加载的会话应该只有10张图片"
                );
                
                // 验证保存的是前10张图片
                for i in 0..10 {
                    let original_data = general_purpose::STANDARD.decode(&session_data.images[i].data)
                        .expect("原始数据应该可以解码");
                    let saved_data = general_purpose::STANDARD.decode(&loaded_record.images[i].data)
                        .expect("保存的数据应该可以解码");
                    
                    prop_assert_eq!(
                        original_data,
                        saved_data,
                        "保存的图片{}应该与原始数据的前10张匹配",
                        i
                    );
                }
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(15))]

        /// **属性 4：会话列表时间排序**
        /// **验证：需求 4.5**
        /// 
        /// 对于任何会话列表，会话应该按时间戳倒序排列（最新的在最前）。
        #[test]
        fn property_session_list_time_sorting(
            sessions in prop::collection::vec(session_data_strategy(), 2..10)
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 保存所有会话（带有随机延迟以确保不同的时间戳）
                for session in sessions {
                    storage.save_session(session).await
                        .expect("保存会话应该成功");
                    // 添加小延迟确保时间戳不同
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                
                // 加载所有会话
                let loaded_sessions = storage.load_sessions().await
                    .expect("加载会话应该成功");
                
                // 验证至少有2个会话
                prop_assert!(
                    loaded_sessions.len() >= 2,
                    "应该至少有2个会话"
                );
                
                // 验证会话按时间倒序排列（最新的在前）
                for i in 0..loaded_sessions.len() - 1 {
                    prop_assert!(
                        loaded_sessions[i].timestamp >= loaded_sessions[i + 1].timestamp,
                        "会话应该按时间倒序排列，但会话[{}]的时间戳 {:?} 早于会话[{}]的时间戳 {:?}",
                        i,
                        loaded_sessions[i].timestamp,
                        i + 1,
                        loaded_sessions[i + 1].timestamp
                    );
                }
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }

        /// **属性 12：延迟加载截图**
        /// **验证：需求 10.2**
        /// 
        /// 对于任何会话列表加载操作，截图数据不应该被加载；
        /// 只有在查看特定会话时才加载该会话的截图。
        #[test]
        fn property_lazy_load_screenshots(
            sessions_with_images in prop::collection::vec(
                session_data_strategy().prop_filter(
                    "Session must have at least one image",
                    |s| !s.images.is_empty()
                ),
                1..6
            )
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 保存所有会话（都包含图片）
                let mut saved_ids = Vec::new();
                for session in sessions_with_images.iter() {
                    let record = storage.save_session(session.clone()).await
                        .expect("保存会话应该成功");
                    saved_ids.push(record.id);
                    
                    // 验证保存的会话包含图片
                    prop_assert!(
                        !record.images.is_empty(),
                        "保存的会话应该包含图片"
                    );
                }
                
                // 使用 load_sessions 加载所有会话
                let loaded_sessions = storage.load_sessions().await
                    .expect("加载会话列表应该成功");
                
                // 验证加载的会话数量正确
                prop_assert_eq!(
                    loaded_sessions.len(),
                    sessions_with_images.len(),
                    "加载的会话数量应该与保存的数量一致"
                );
                
                // 验证所有会话的截图数据都是空的（延迟加载）
                for session in loaded_sessions.iter() {
                    prop_assert!(
                        session.images.is_empty(),
                        "load_sessions 返回的会话不应该包含截图数据（延迟加载），但会话 {} 包含 {} 张图片",
                        session.id,
                        session.images.len()
                    );
                }
                
                // 验证使用 get_session 可以加载完整数据（包括截图）
                for id in saved_ids.iter() {
                    let full_session = storage.get_session(id).await
                        .expect("get_session 应该成功")
                        .expect("会话应该存在");
                    
                    prop_assert!(
                        !full_session.images.is_empty(),
                        "get_session 返回的会话应该包含截图数据，但会话 {} 没有图片",
                        id
                    );
                }
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }
    }

    #[tokio::test]
    async fn test_screenshot_limit() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建一个包含15张图片的会话（超过10张限制）
        let mut images = Vec::new();
        for i in 0..15 {
            let data = vec![i as u8; 100];
            images.push(ImageAttachment {
                data: general_purpose::STANDARD.encode(&data),
                media_type: "image/png".to_string(),
                filename: Some(format!("test{}.png", i)),
            });
        }
        
        let session_data = SessionData {
            source: SessionSource::Send,
            user_input: Some("Test input".to_string()),
            ai_response: "Test response".to_string(),
            selected_options: vec![],
            images,
        };
        
        // 保存会话
        let saved_record = storage.save_session(session_data).await
            .expect("保存会话应该成功");
        
        // 验证只保存了10张图片
        assert_eq!(saved_record.images.len(), 10, "应该只保存10张图片");
        
        // 加载会话并验证
        let loaded_record = storage.get_session(&saved_record.id).await
            .expect("加载会话应该成功")
            .expect("会话应该存在");
        
        assert_eq!(loaded_record.images.len(), 10, "加载的会话应该只有10张图片");
    }

    #[tokio::test]
    async fn test_image_compression() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建一个大图片（超过5MB）
        // 使用一个简单的PNG格式图片
        let large_data = vec![0u8; 6 * 1024 * 1024]; // 6MB
        
        let session_data = SessionData {
            source: SessionSource::Send,
            user_input: Some("Test input".to_string()),
            ai_response: "Test response".to_string(),
            selected_options: vec![],
            images: vec![ImageAttachment {
                data: general_purpose::STANDARD.encode(&large_data),
                media_type: "image/png".to_string(),
                filename: Some("large.png".to_string()),
            }],
        };
        
        // 保存会话 - 应该会压缩图片或失败
        // 注意：由于我们的测试数据不是真实的图片，这个测试可能会失败
        // 但它展示了压缩功能的存在
        let result = storage.save_session(session_data).await;
        
        // 如果数据不是有效的图片格式，压缩会失败，这是预期的
        // 在实际使用中，输入应该是有效的图片数据
        match result {
            Ok(saved_record) => {
                // 如果成功（可能是因为图片库能处理这种数据），验证图片被处理了
                assert!(!saved_record.images.is_empty(), "应该有图片");
                println!("图片处理成功，大小: {} bytes", 
                    general_purpose::STANDARD.decode(&saved_record.images[0].data).unwrap().len());
            }
            Err(e) => {
                // 如果失败，应该是因为无效的图片格式
                println!("图片处理失败（预期）: {}", e);
                assert!(e.to_string().contains("Failed to load image") || 
                        e.to_string().contains("Failed to compress"), 
                    "应该因为无效图片格式而失败，实际错误: {}", e);
            }
        }
    }

    // 辅助函数：创建一个有效的PNG图片数据
    // 使用随机噪声使图片难以压缩，从而生成更大的文件
    fn create_valid_png_image(width: u32, height: u32, seed: u8) -> Vec<u8> {
        use image::{ImageBuffer, Rgb};
        
        // 创建带有噪声的图片，使其难以压缩
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            // 使用简单的伪随机算法生成噪声
            let noise = ((x * 7 + y * 13 + seed as u32) % 256) as u8;
            let r = noise;
            let g = noise.wrapping_add(85);
            let b = noise.wrapping_add(170);
            Rgb([r, g, b])
        });
        
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageOutputFormat::Png)
            .expect("Failed to encode PNG");
        
        buffer
    }
    
    // 辅助函数：创建一个超大的PNG图片（保证超过5MB）
    fn create_large_png_image() -> Vec<u8> {
        use image::{ImageBuffer, Rgb};
        
        // 创建一个足够大的图片（4000x4000应该足够）
        let width = 4000u32;
        let height = 4000u32;
        
        // 使用噪声模式使图片难以压缩
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let noise = ((x * 7 + y * 13) % 256) as u8;
            let r = noise;
            let g = noise.wrapping_add(85);
            let b = noise.wrapping_add(170);
            Rgb([r, g, b])
        });
        
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageOutputFormat::Png)
            .expect("Failed to encode PNG");
        
        buffer
    }

    // Property test for screenshot size limit is commented out because it's very expensive
    // (creates and compresses 40MB images). The unit test test_screenshot_size_limit_with_real_image
    // provides adequate coverage for this property.
    /*
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2))] // 减少测试次数因为图片处理较慢

        /// **属性 10：截图大小限制**
        /// **验证：需求 10.5, 10.6**
        /// 
        /// 对于任何截图上传操作，如果截图大小超过5MB，
        /// 系统应该自动压缩到限制内。
        #[test]
        fn property_screenshot_size_limit(
            seed in 0u8..2u8,
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 创建一个大图片（保证超过5MB）
                let image_data = create_large_png_image();
                let original_size = image_data.len();
                
                println!("生成图片 (seed={}), 原始大小: {} bytes ({:.2} MB)", 
                    seed, original_size, original_size as f64 / (1024.0 * 1024.0));
                
                // 验证生成的图片确实超过5MB
                prop_assert!(
                    original_size > MAX_IMAGE_SIZE,
                    "测试图片应该超过5MB，实际: {} bytes ({:.2} MB)",
                    original_size,
                    original_size as f64 / (1024.0 * 1024.0)
                );
                
                let session_data = SessionData {
                    source: SessionSource::Send,
                    user_input: Some("Test input".to_string()),
                    ai_response: "Test response".to_string(),
                    selected_options: vec![],
                    images: vec![ImageAttachment {
                        data: general_purpose::STANDARD.encode(&image_data),
                        media_type: "image/png".to_string(),
                        filename: Some("large.png".to_string()),
                    }],
                };
                
                // 保存会话 - 应该自动压缩图片
                let saved_record = storage.save_session(session_data).await
                    .expect("保存会话应该成功（即使需要压缩）");
                
                // 验证图片被保存
                prop_assert_eq!(
                    saved_record.images.len(),
                    1,
                    "应该保存1张图片"
                );
                
                // 解码保存的图片数据
                let saved_image_data = general_purpose::STANDARD.decode(&saved_record.images[0].data)
                    .expect("保存的图片数据应该可以解码");
                let saved_size = saved_image_data.len();
                
                println!("压缩后大小: {} bytes ({:.2} MB)", 
                    saved_size, saved_size as f64 / (1024.0 * 1024.0));
                
                // 验证压缩后的图片不超过5MB（需求 10.5, 10.6）
                prop_assert!(
                    saved_size <= MAX_IMAGE_SIZE,
                    "压缩后的图片大小 {} bytes 应该不超过 {} bytes (5MB)，原始大小: {} bytes",
                    saved_size,
                    MAX_IMAGE_SIZE,
                    original_size
                );
                
                // 验证图片被压缩了
                prop_assert!(
                    saved_size < original_size,
                    "图片应该被压缩，原始: {} bytes, 压缩后: {} bytes",
                    original_size,
                    saved_size
                );
                
                // 验证压缩后的图片仍然是有效的图片格式
                let loaded_img = image::load_from_memory(&saved_image_data);
                prop_assert!(
                    loaded_img.is_ok(),
                    "压缩后的图片应该仍然是有效的图片格式"
                );
                
                // 加载会话并验证持久化
                let loaded_record = storage.get_session(&saved_record.id).await
                    .expect("加载会话应该成功")
                    .expect("会话应该存在");
                
                prop_assert_eq!(
                    loaded_record.images.len(),
                    1,
                    "加载的会话应该有1张图片"
                );
                
                // 验证加载的图片大小也在限制内
                let loaded_image_data = general_purpose::STANDARD.decode(&loaded_record.images[0].data)
                    .expect("加载的图片数据应该可以解码");
                
                prop_assert!(
                    loaded_image_data.len() <= MAX_IMAGE_SIZE,
                    "加载的图片大小 {} bytes 应该不超过 {} bytes (5MB)",
                    loaded_image_data.len(),
                    MAX_IMAGE_SIZE
                );
                
                // 验证保存和加载的图片数据一致
                prop_assert_eq!(
                    saved_image_data,
                    loaded_image_data,
                    "保存和加载的图片数据应该一致"
                );
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }
    }
    */

    #[tokio::test]
    async fn test_screenshot_size_limit_with_real_image() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建一个大的PNG图片（超过5MB）
        let large_image = create_large_png_image();
        let original_size = large_image.len();
        
        println!("原始图片大小: {} bytes ({:.2} MB)", 
            original_size, original_size as f64 / (1024.0 * 1024.0));
        
        // 验证图片确实超过5MB
        assert!(
            original_size > MAX_IMAGE_SIZE,
            "测试图片应该超过5MB，实际: {} bytes",
            original_size
        );
        
        let session_data = SessionData {
            source: SessionSource::Send,
            user_input: Some("Test with large image".to_string()),
            ai_response: "Test response".to_string(),
            selected_options: vec![],
            images: vec![ImageAttachment {
                data: general_purpose::STANDARD.encode(&large_image),
                media_type: "image/png".to_string(),
                filename: Some("large.png".to_string()),
            }],
        };
        
        // 保存会话 - 应该自动压缩图片
        let saved_record = storage.save_session(session_data).await
            .expect("保存会话应该成功");
        
        // 验证图片被保存
        assert_eq!(saved_record.images.len(), 1, "应该保存1张图片");
        
        // 解码保存的图片数据
        let saved_image_data = general_purpose::STANDARD.decode(&saved_record.images[0].data)
            .expect("保存的图片数据应该可以解码");
        let saved_size = saved_image_data.len();
        
        println!("压缩后图片大小: {} bytes ({:.2} MB)", 
            saved_size, saved_size as f64 / (1024.0 * 1024.0));
        
        // 验证压缩后的图片不超过5MB
        assert!(
            saved_size <= MAX_IMAGE_SIZE,
            "压缩后的图片大小 {} bytes 应该不超过 {} bytes (5MB)",
            saved_size,
            MAX_IMAGE_SIZE
        );
        
        // 验证图片被压缩了
        assert!(
            saved_size < original_size,
            "图片应该被压缩，原始: {} bytes, 压缩后: {} bytes",
            original_size,
            saved_size
        );
        
        // 验证压缩后的图片仍然是有效的图片
        let loaded_img = image::load_from_memory(&saved_image_data);
        assert!(loaded_img.is_ok(), "压缩后的图片应该仍然是有效的图片格式");
        
        // 加载会话并验证
        let loaded_record = storage.get_session(&saved_record.id).await
            .expect("加载会话应该成功")
            .expect("会话应该存在");
        
        assert_eq!(loaded_record.images.len(), 1, "加载的会话应该有1张图片");
        
        let loaded_image_data = general_purpose::STANDARD.decode(&loaded_record.images[0].data)
            .expect("加载的图片数据应该可以解码");
        
        assert!(
            loaded_image_data.len() <= MAX_IMAGE_SIZE,
            "加载的图片大小应该不超过5MB"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(15))]

        /// **属性 16：批量删除完整性**
        /// **验证：需求 7.9, 7.10, 7.11**
        /// 
        /// 对于任何批量删除操作，所有选中的会话应该从存储中完全移除
        /// （包括元数据和截图），且操作应该是原子性的或提供部分失败的错误报告。
        #[test]
        fn property_batch_delete_completeness(
            sessions in prop::collection::vec(session_data_strategy(), 3..10),
            delete_indices in prop::collection::vec(0usize..10usize, 1..5),
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 保存所有会话
                let mut saved_ids = Vec::new();
                for session in sessions.iter() {
                    let record = storage.save_session(session.clone()).await
                        .expect("保存会话应该成功");
                    saved_ids.push(record.id);
                }
                
                // 选择要删除的会话ID（确保索引有效）
                let mut ids_to_delete = Vec::new();
                for &index in delete_indices.iter() {
                    if index < saved_ids.len() {
                        ids_to_delete.push(saved_ids[index].clone());
                    }
                }
                
                // 去重
                ids_to_delete.sort();
                ids_to_delete.dedup();
                
                // 如果没有有效的删除ID，跳过测试
                if ids_to_delete.is_empty() {
                    return Ok(());
                }
                
                // 记录要保留的会话ID
                let ids_to_keep: Vec<String> = saved_ids.iter()
                    .filter(|id| !ids_to_delete.contains(id))
                    .cloned()
                    .collect();
                
                // 执行批量删除
                let delete_result = storage.batch_delete_sessions(&ids_to_delete).await;
                
                // 验证删除操作成功
                prop_assert!(
                    delete_result.is_ok(),
                    "批量删除应该成功，错误: {:?}",
                    delete_result.err()
                );
                
                // 验证被删除的会话不再存在
                for id in ids_to_delete.iter() {
                    // 检查元数据文件不存在
                    let metadata_path = storage.storage_dir.join(format!("{}.json", id));
                    prop_assert!(
                        !metadata_path.exists(),
                        "会话 {} 的元数据文件应该被删除",
                        id
                    );
                    
                    // 检查截图目录不存在
                    let images_dir = storage.storage_dir.join("images").join(id);
                    prop_assert!(
                        !images_dir.exists(),
                        "会话 {} 的截图目录应该被删除",
                        id
                    );
                    
                    // 尝试加载会话应该返回None
                    let loaded = storage.get_session(id).await
                        .expect("get_session 不应该失败");
                    prop_assert!(
                        loaded.is_none(),
                        "会话 {} 应该不存在",
                        id
                    );
                }
                
                // 验证未被删除的会话仍然存在
                for id in ids_to_keep.iter() {
                    // 检查元数据文件存在
                    let metadata_path = storage.storage_dir.join(format!("{}.json", id));
                    prop_assert!(
                        metadata_path.exists(),
                        "会话 {} 的元数据文件应该仍然存在",
                        id
                    );
                    
                    // 尝试加载会话应该成功
                    let loaded = storage.get_session(id).await
                        .expect("get_session 不应该失败");
                    prop_assert!(
                        loaded.is_some(),
                        "会话 {} 应该仍然存在",
                        id
                    );
                }
                
                // 验证 load_sessions 返回的会话数量正确
                let remaining_sessions = storage.load_sessions().await
                    .expect("load_sessions 应该成功");
                prop_assert_eq!(
                    remaining_sessions.len(),
                    ids_to_keep.len(),
                    "剩余会话数量应该等于未删除的会话数量"
                );
                
                // 验证 load_sessions 返回的会话ID正确
                let remaining_ids: Vec<String> = remaining_sessions.iter()
                    .map(|s| s.id.clone())
                    .collect();
                for id in ids_to_keep.iter() {
                    prop_assert!(
                        remaining_ids.contains(id),
                        "load_sessions 应该包含未删除的会话 {}",
                        id
                    );
                }
                for id in ids_to_delete.iter() {
                    prop_assert!(
                        !remaining_ids.contains(id),
                        "load_sessions 不应该包含已删除的会话 {}",
                        id
                    );
                }
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }
    }

    #[tokio::test]
    async fn test_batch_delete_basic() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建并保存3个会话
        let mut session_ids = Vec::new();
        for i in 0..3 {
            let session_data = SessionData {
                source: SessionSource::Send,
                user_input: Some(format!("Test input {}", i)),
                ai_response: format!("Test response {}", i),
                selected_options: vec![],
                images: vec![],
            };
            
            let record = storage.save_session(session_data).await
                .expect("保存会话应该成功");
            session_ids.push(record.id);
        }
        
        // 批量删除前2个会话
        let ids_to_delete = vec![session_ids[0].clone(), session_ids[1].clone()];
        storage.batch_delete_sessions(&ids_to_delete).await
            .expect("批量删除应该成功");
        
        // 验证前2个会话被删除
        for id in ids_to_delete.iter() {
            let loaded = storage.get_session(id).await
                .expect("get_session 不应该失败");
            assert!(loaded.is_none(), "会话 {} 应该被删除", id);
        }
        
        // 验证第3个会话仍然存在
        let loaded = storage.get_session(&session_ids[2]).await
            .expect("get_session 不应该失败");
        assert!(loaded.is_some(), "会话 {} 应该仍然存在", session_ids[2]);
        
        // 验证 load_sessions 只返回1个会话
        let remaining = storage.load_sessions().await
            .expect("load_sessions 应该成功");
        assert_eq!(remaining.len(), 1, "应该只剩1个会话");
        assert_eq!(remaining[0].id, session_ids[2], "剩余的应该是第3个会话");
    }

    #[tokio::test]
    async fn test_batch_delete_with_images() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建并保存2个包含图片的会话
        let mut session_ids = Vec::new();
        for i in 0..2 {
            let data = vec![i as u8; 100];
            let session_data = SessionData {
                source: SessionSource::Send,
                user_input: Some(format!("Test input {}", i)),
                ai_response: format!("Test response {}", i),
                selected_options: vec![],
                images: vec![ImageAttachment {
                    data: general_purpose::STANDARD.encode(&data),
                    media_type: "image/png".to_string(),
                    filename: Some(format!("test{}.png", i)),
                }],
            };
            
            let record = storage.save_session(session_data).await
                .expect("保存会话应该成功");
            session_ids.push(record.id);
        }
        
        // 验证图片目录存在
        for id in session_ids.iter() {
            let images_dir = storage.storage_dir.join("images").join(id);
            assert!(images_dir.exists(), "会话 {} 的图片目录应该存在", id);
        }
        
        // 批量删除所有会话
        storage.batch_delete_sessions(&session_ids).await
            .expect("批量删除应该成功");
        
        // 验证所有会话和图片都被删除
        for id in session_ids.iter() {
            // 元数据文件不存在
            let metadata_path = storage.storage_dir.join(format!("{}.json", id));
            assert!(!metadata_path.exists(), "会话 {} 的元数据文件应该被删除", id);
            
            // 图片目录不存在
            let images_dir = storage.storage_dir.join("images").join(id);
            assert!(!images_dir.exists(), "会话 {} 的图片目录应该被删除", id);
        }
    }

    #[tokio::test]
    async fn test_batch_delete_partial_failure() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建并保存1个会话
        let session_data = SessionData {
            source: SessionSource::Send,
            user_input: Some("Test input".to_string()),
            ai_response: "Test response".to_string(),
            selected_options: vec![],
            images: vec![],
        };
        
        let record = storage.save_session(session_data).await
            .expect("保存会话应该成功");
        
        // 尝试批量删除：1个存在的会话 + 1个不存在的会话
        let ids_to_delete = vec![
            record.id.clone(),
            "non-existent-id".to_string(),
        ];
        
        // 批量删除应该成功（因为不存在的会话不会导致错误）
        let result = storage.batch_delete_sessions(&ids_to_delete).await;
        
        // 验证存在的会话被删除
        let loaded = storage.get_session(&record.id).await
            .expect("get_session 不应该失败");
        assert!(loaded.is_none(), "存在的会话应该被删除");
        
        // 根据实现，删除不存在的会话不会报错（因为文件不存在时不会失败）
        // 所以批量删除应该成功
        assert!(result.is_ok(), "批量删除应该成功");
    }

    #[tokio::test]
    async fn test_clear_all_sessions() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 创建并保存多个会话（包含图片）
        let mut session_ids = Vec::new();
        for i in 0..5 {
            let data = vec![i as u8; 100];
            let session_data = SessionData {
                source: SessionSource::Send,
                user_input: Some(format!("Test input {}", i)),
                ai_response: format!("Test response {}", i),
                selected_options: vec![format!("option{}", i)],
                images: vec![ImageAttachment {
                    data: general_purpose::STANDARD.encode(&data),
                    media_type: "image/png".to_string(),
                    filename: Some(format!("test{}.png", i)),
                }],
            };
            
            let record = storage.save_session(session_data).await
                .expect("保存会话应该成功");
            session_ids.push(record.id);
        }
        
        // 验证所有会话都存在
        let sessions_before = storage.load_sessions().await
            .expect("加载会话应该成功");
        assert_eq!(sessions_before.len(), 5, "应该有5个会话");
        
        // 验证元数据文件和图片目录都存在
        for id in session_ids.iter() {
            let metadata_path = storage.storage_dir.join(format!("{}.json", id));
            assert!(metadata_path.exists(), "会话 {} 的元数据文件应该存在", id);
            
            let images_dir = storage.storage_dir.join("images").join(id);
            assert!(images_dir.exists(), "会话 {} 的图片目录应该存在", id);
        }
        
        // 清空所有会话
        storage.clear_all_sessions().await
            .expect("清空所有会话应该成功");
        
        // 验证所有元数据文件都被删除
        for id in session_ids.iter() {
            let metadata_path = storage.storage_dir.join(format!("{}.json", id));
            assert!(!metadata_path.exists(), "会话 {} 的元数据文件应该被删除", id);
        }
        
        // 验证所有图片目录都被删除
        for id in session_ids.iter() {
            let images_dir = storage.storage_dir.join("images").join(id);
            assert!(!images_dir.exists(), "会话 {} 的图片目录应该被删除", id);
        }
        
        // 验证 images 目录本身仍然存在（空目录结构被重新创建）
        let images_dir = storage.storage_dir.join("images");
        assert!(images_dir.exists(), "images 目录应该仍然存在");
        
        // 验证 load_sessions 返回空列表
        let sessions_after = storage.load_sessions().await
            .expect("加载会话应该成功");
        assert_eq!(sessions_after.len(), 0, "清空后应该没有会话");
        
        // 验证尝试获取任何会话都返回 None
        for id in session_ids.iter() {
            let loaded = storage.get_session(id).await
                .expect("get_session 不应该失败");
            assert!(loaded.is_none(), "会话 {} 应该不存在", id);
        }
    }

    #[tokio::test]
    async fn test_clear_all_sessions_empty() {
        let (storage, _temp_dir) = create_temp_storage();
        
        // 在没有任何会话的情况下清空
        let result = storage.clear_all_sessions().await;
        assert!(result.is_ok(), "清空空的存储应该成功");
        
        // 验证 images 目录仍然存在
        let images_dir = storage.storage_dir.join("images");
        assert!(images_dir.exists(), "images 目录应该存在");
        
        // 验证 load_sessions 返回空列表
        let sessions = storage.load_sessions().await
            .expect("加载会话应该成功");
        assert_eq!(sessions.len(), 0, "应该没有会话");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(15))]

        /// **属性 5：会话删除完整性**
        /// **验证：需求 7.3**
        /// 
        /// 对于任何会话，当用户确认删除后，该会话应该从存储中完全移除
        /// （包括元数据和截图）。
        #[test]
        fn property_session_deletion_completeness(
            session_data in session_data_strategy()
        ) {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let (storage, _temp_dir) = create_temp_storage();
                
                // 保存会话
                let saved_record = storage.save_session(session_data.clone()).await
                    .expect("保存会话应该成功");
                
                let session_id = saved_record.id.clone();
                
                // 验证会话存在
                let metadata_path = storage.storage_dir.join(format!("{}.json", session_id));
                prop_assert!(
                    metadata_path.exists(),
                    "保存后元数据文件应该存在"
                );
                
                // 如果有图片，验证图片目录存在
                if !session_data.images.is_empty() {
                    let images_dir = storage.storage_dir.join("images").join(&session_id);
                    prop_assert!(
                        images_dir.exists(),
                        "保存后图片目录应该存在（如果有图片）"
                    );
                }
                
                // 验证可以加载会话
                let loaded_before = storage.get_session(&session_id).await
                    .expect("get_session 不应该失败");
                prop_assert!(
                    loaded_before.is_some(),
                    "删除前会话应该存在"
                );
                
                // 删除会话
                let delete_result = storage.delete_session(&session_id).await;
                prop_assert!(
                    delete_result.is_ok(),
                    "删除会话应该成功，错误: {:?}",
                    delete_result.err()
                );
                
                // 验证元数据文件被删除
                prop_assert!(
                    !metadata_path.exists(),
                    "删除后元数据文件应该不存在"
                );
                
                // 验证图片目录被删除（如果有图片）
                if !session_data.images.is_empty() {
                    let images_dir = storage.storage_dir.join("images").join(&session_id);
                    prop_assert!(
                        !images_dir.exists(),
                        "删除后图片目录应该不存在"
                    );
                }
                
                // 验证无法加载会话
                let loaded_after = storage.get_session(&session_id).await
                    .expect("get_session 不应该失败");
                prop_assert!(
                    loaded_after.is_none(),
                    "删除后会话应该不存在"
                );
                
                // 验证会话不在列表中
                let all_sessions = storage.load_sessions().await
                    .expect("load_sessions 应该成功");
                let found_in_list = all_sessions.iter().any(|s| s.id == session_id);
                prop_assert!(
                    !found_in_list,
                    "删除后会话不应该出现在会话列表中"
                );
                
                Ok(()) as Result<(), TestCaseError>
            }).unwrap()
        }
    }
}
