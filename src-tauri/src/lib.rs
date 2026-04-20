pub mod discovery;
pub mod transfer;
pub mod history;
pub mod confirm;

use tauri::{command, AppHandle};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    success: bool,
    message: String,
    transferred_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadStatusResult {
    success: bool,
    exists: bool,
    received_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    id: String,
    name: String,
    path: String,
    size: u64,
    relative_path: String,
    is_dir: bool,
}

// 获取本机局域网IP的命令
#[command]
fn get_local_ip() -> String {
    discovery::get_local_ip()
}

// 遍历文件夹获取文件列表
#[command]
fn list_folder_files(folder_path: String) -> Vec<FileEntry> {
    let folder = PathBuf::from(&folder_path);
    let folder_name = folder.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut files = Vec::new();
    let mut index = 0;

    for entry in WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let relative_path = path.strip_prefix(&folder)
                .map(|p| format!("{}\\{}", folder_name, p.to_string_lossy()))
                .unwrap_or_else(|_| name.clone());

            files.push(FileEntry {
                id: format!("file_{}", index),
                name,
                path: path.to_string_lossy().to_string(),
                size,
                relative_path,
                is_dir: false,
            });
            index += 1;
        }
    }

    files
}

// 获取文件大小
#[command]
fn get_file_size(file_path: String) -> u64 {
    std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0)
}

// 获取传输历史记录
#[command]
fn get_transfer_history(app_handle: AppHandle) -> Vec<history::models::TransferRecord> {
    let conn = history::database::get_connection(&app_handle);
    history::database::get_history(&conn)
}

// 保存传输记录
#[command]
fn save_transfer_record(
    app_handle: AppHandle,
    id: String,
    direction: String,
    peer_device_id: String,
    peer_device_name: String,
    total_size: u64,
) {
    let conn = history::database::get_connection(&app_handle);
    history::database::add_transfer(&conn, &id, &direction, &peer_device_id, &peer_device_name, total_size);
}

// 更新传输状态
#[command]
fn update_transfer_status(
    app_handle: AppHandle,
    id: String,
    status: String,
    transferred_size: u64,
) {
    let conn = history::database::get_connection(&app_handle);
    history::database::update_status(&conn, &id, &status, transferred_size);
}

// 查询已上传文件大小（断点续传）
#[command]
async fn query_upload_status(
    receiver_ip: String,
    receiver_port: u16,
    transfer_id: String,
    file_name: String,
    relative_path: String,
) -> UploadStatusResult {
    let url = format!("http://{}:{}{}", receiver_ip, receiver_port, "/api/upload/status");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id,
        "file_name": file_name,
        "relative_path": relative_path
    });

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(_) => {
            return UploadStatusResult {
                success: false,
                exists: false,
                received_size: 0,
            };
        }
    };

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await.unwrap_or(serde_json::json!({}));
        UploadStatusResult {
            success: true,
            exists: result["exists"].as_bool().unwrap_or(false),
            received_size: result["received_size"].as_u64().unwrap_or(0),
        }
    } else {
        UploadStatusResult {
            success: false,
            exists: false,
            received_size: 0,
        }
    }
}

// 上传文件到接收方（支持断点续传）
#[command]
async fn upload_file_to_receiver(
    file_path: String,
    transfer_id: String,
    file_id: String,
    file_name: String,
    relative_path: Option<String>,
    receiver_ip: String,
    receiver_port: u16,
    offset: Option<u64>,
) -> UploadResult {
    let path = PathBuf::from(&file_path);
    let start_offset = offset.unwrap_or(0);

    // 读取文件内容
    let file_content = match tokio::fs::read(&path).await {
        Ok(content) => content,
        Err(e) => {
            return UploadResult {
                success: false,
                message: format!("无法读取文件: {}", e),
                transferred_size: 0,
            };
        }
    };

    // 断点续传：只发送未传输的部分
    let content_to_send = if start_offset > 0 && start_offset < file_content.len() as u64 {
        &file_content[start_offset as usize..]
    } else {
        &file_content[..]
    };

    let send_size = content_to_send.len() as u64;

    // 构建 multipart 表单
    let file_name_clone = file_name.clone();
    let form = reqwest::multipart::Form::new()
        .text("transfer_id", transfer_id)
        .text("file_id", file_id)
        .text("file_name", file_name)
        .text("offset", start_offset.to_string())
        .part("file", reqwest::multipart::Part::bytes(content_to_send.to_vec())
            .file_name(file_name_clone));

    let form = if let Some(rp) = relative_path {
        form.text("relative_path", rp)
    } else {
        form
    };

    // 发送上传请求
    let url = format!("http://{}:{}{}", receiver_ip, receiver_port, "/api/upload");
    let client = reqwest::Client::new();

    let response = match client.post(&url).multipart(form).send().await {
        Ok(r) => r,
        Err(e) => {
            return UploadResult {
                success: false,
                message: format!("上传请求失败: {}", e),
                transferred_size: 0,
            };
        }
    };

    if response.status().is_success() {
        UploadResult {
            success: true,
            message: "上传成功".to_string(),
            transferred_size: start_offset + send_size,
        }
    } else {
        UploadResult {
            success: false,
            message: format!("上传失败: {}", response.status()),
            transferred_size: 0,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_local_ip,
            upload_file_to_receiver,
            query_upload_status,
            get_transfer_history,
            save_transfer_record,
            update_transfer_status,
            list_folder_files,
            get_file_size
        ])
        .setup(|app| {
            // 启动设备发现服务
            let app_handle = app.handle();
            discovery::start_discovery(app_handle.clone());

            // 启动HTTP传输服务
            transfer::start_http_server(app_handle.clone());

            // 初始化历史记录数据库
            history::init_database(app_handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}