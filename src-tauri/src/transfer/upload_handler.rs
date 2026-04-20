use axum::{
    extract::{State, Multipart},
    Json,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncSeekExt, SeekFrom};
use tauri::{Emitter, Manager};

use super::http_server::AppState;
use super::http_server::is_transfer_cancelled;

/// 处理文件上传（支持断点续传）
pub async fn handle_upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    let mut transfer_id = String::new();
    let mut file_id = String::new();
    let mut file_name = String::new();
    let mut relative_path = String::new();
    let mut offset: u64 = 0; // 断点续传起始位置
    let mut received_size: u64 = 0;
    let mut file_data: Option<Vec<u8>> = None;

    // 先解析所有字段
    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "transfer_id" => {
                transfer_id = field.text().await.unwrap_or_default();
            }
            "file_id" => {
                file_id = field.text().await.unwrap_or_default();
            }
            "file_name" => {
                file_name = field.text().await.unwrap_or_default();
            }
            "relative_path" => {
                relative_path = field.text().await.unwrap_or_default();
            }
            "offset" => {
                // 断点续传起始位置
                offset = field.text().await.unwrap_or_default().parse().unwrap_or(0);
            }
            "file" => {
                // 收集文件数据
                let mut data = Vec::new();
                while let Some(chunk) = field.chunk().await.unwrap_or(None) {
                    data.extend_from_slice(&chunk);
                    received_size += chunk.len() as u64;
                }
                file_data = Some(data);
            }
            _ => {}
        }
    }

    // 检查传输是否已取消
    if is_transfer_cancelled(&state, &transfer_id).await {
        return Json(serde_json::json!({
            "success": false,
            "message": "传输已取消",
            "file_id": file_id,
            "transfer_id": transfer_id
        }));
    }

    // 获取保存路径
    let save_path = {
        let paths = state.save_paths.lock().await;
        paths.get(&transfer_id).cloned().unwrap_or_default()
    };

    // 实际保存的文件路径
    let mut actual_file_path: Option<String> = None;

    // 写入文件
    if let Some(data) = file_data {
        // 确定保存路径
        let base_path = if save_path.is_empty() {
            let app_dir = state.app_handle.path().app_data_dir().unwrap_or_default();
            app_dir.join("transfers").join(&transfer_id)
        } else {
            std::path::PathBuf::from(&save_path)
        };

        // 根据relative_path确定最终文件路径
        let file_path = if relative_path.is_empty() {
            base_path.join(&file_name)
        } else {
            base_path.join(&relative_path)
        };

        // 创建目录
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or(());
        }

        actual_file_path = Some(file_path.to_string_lossy().to_string());

        // 断点续传：追加写入或从头写入
        let mut file = if offset > 0 && file_path.exists() {
            // 断点续传模式：追加写入
            let mut f = File::options()
                .write(true)
                .open(&file_path)
                .await
                .unwrap();
            f.seek(SeekFrom::Start(offset)).await.unwrap();
            f
        } else {
            // 新文件或从头开始
            File::create(&file_path).await.unwrap()
        };

        file.write_all(&data).await.unwrap();
    }

    // 总接收大小 = 断点位置 + 本次接收大小
    let total_received = offset + received_size;

    println!("文件上传完成: transfer_id={}, file_id={}, file_name={}, received_size={}", transfer_id, file_id, file_name, total_received);

    // 保存文件记录到数据库（首次上传时）
    if offset == 0 && actual_file_path.is_some() {
        let conn = crate::history::database::get_connection(&state.app_handle);
        crate::history::database::add_file_record(
            &conn,
            &file_id,
            &transfer_id,
            &file_name,
            actual_file_path.as_deref(),
            total_received,
            None,  // md5
            "received",
        );
    }

    // 发送进度更新事件到前端（通过window发送）
    if let Some(window) = state.app_handle.get_webview_window("main") {
        let emit_result = window.emit("upload-progress", serde_json::json!({
            "transfer_id": transfer_id,
            "file_id": file_id,
            "received_size": total_received
        }));
        println!("upload-progress事件发送结果: {:?}", emit_result);
    } else {
        println!("未找到main window，无法发送upload-progress事件");
    }

    Json(serde_json::json!({
        "success": true,
        "file_id": file_id,
        "transfer_id": transfer_id,
        "received_size": total_received,
        "offset": offset
    }))
}