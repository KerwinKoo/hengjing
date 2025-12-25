use crate::config::AppState;
use crate::log_important;
use tauri::{AppHandle, Manager, WindowEvent};
use std::sync::atomic::{AtomicBool, Ordering};

// 防止重复处理关闭事件
static CLOSE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// 设置窗口事件监听器
pub fn setup_window_event_listeners(app_handle: &AppHandle) {
    log_important!(info, "设置窗口事件监听器");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        let app_handle_clone = app_handle.clone();
        
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 阻止默认的关闭行为
                api.prevent_close();
                
                // 检查是否已经在处理关闭事件
                if CLOSE_IN_PROGRESS.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                    // 已经在处理中，忽略重复事件
                    return;
                }
                
                let app_handle = app_handle_clone.clone();
                
                // 异步处理退出请求
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<AppState>();

                    log_important!(info, "窗口关闭按钮被点击");

                    // 窗口关闭按钮点击应该直接退出，不需要双重确认
                    match crate::ui::exit::handle_system_exit_request(
                        state,
                        &app_handle,
                        true, // 手动点击关闭按钮
                    ).await {
                        Ok(exited) => {
                            if !exited {
                                log_important!(info, "退出被阻止，等待二次确认");
                                // 重置标志，允许下次点击
                                CLOSE_IN_PROGRESS.store(false, Ordering::SeqCst);
                            }
                        }
                        Err(e) => {
                            log_important!(error, "处理退出请求失败: {}", e);
                            // 重置标志
                            CLOSE_IN_PROGRESS.store(false, Ordering::SeqCst);
                        }
                    }
                });
            }
        });
    } else {
        log_important!(warn, "未找到 main 窗口");
    }
}
