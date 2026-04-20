pub mod discovery;
pub mod transfer;
pub mod history;
pub mod confirm;
pub mod settings;

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

// 获取HTTP端口
#[command]
fn get_http_port() -> u16 {
    transfer::get_http_port()
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

// 计算文件MD5值
#[command]
fn calculate_file_md5(file_path: String) -> Option<String> {
    match std::fs::read(&file_path) {
        Ok(content) => {
            let digest = md5::compute(&content);
            Some(format!("{:x}", digest))
        }
        Err(_) => None
    }
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
    peer_ip: String,
    total_size: u64,
) {
    let conn = history::database::get_connection(&app_handle);
    history::database::add_transfer(&conn, &id, &direction, &peer_device_id, &peer_device_name, &peer_ip, total_size);
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

// 保存文件记录
#[command]
fn save_file_record(
    app_handle: AppHandle,
    id: String,
    transfer_id: String,
    name: String,
    path: Option<String>,
    size: u64,
    md5: Option<String>,
    status: String,
) {
    let conn = history::database::get_connection(&app_handle);
    history::database::add_file_record(&conn, &id, &transfer_id, &name, path.as_deref(), size, md5.as_deref(), &status);
}

// 手动触发设备扫描
#[command]
fn trigger_device_scan() {
    discovery::trigger_scan();
}

// 获取传输的文件列表
#[command]
fn get_transfer_files(app_handle: AppHandle, transfer_id: String) -> Vec<history::models::FileRecord> {
    let conn = history::database::get_connection(&app_handle);
    history::database::get_files_by_transfer(&conn, &transfer_id)
}

// 获取所有设置
#[command]
fn get_settings(app_handle: AppHandle) -> settings::models::AppSettings {
    let conn = history::database::get_connection(&app_handle);
    settings::database::get_all_settings(&conn)
}

// 设置默认保存路径
#[command]
fn set_default_save_path(app_handle: AppHandle, path: String) -> Result<(), String> {
    // 验证路径（非空时）
    if !path.is_empty() {
        settings::database::validate_path(&path)?;
    }

    let conn = history::database::get_connection(&app_handle);
    settings::database::set_setting(&conn, "default_save_path", &path)
        .map_err(|e| e.to_string())
}

// 验证保存路径
#[command]
fn validate_save_path(path: String) -> Result<(), String> {
    settings::database::validate_path(&path)
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransferResult {
    success: bool,
    message: String,
}

// 发送传输请求到目标设备
#[command]
async fn send_transfer_request(
    device_ip: String,
    device_port: u16,
    transfer_id: String,
    sender_name: String,
    sender_ip: String,
    sender_port: u16,
    files: Vec<serde_json::Value>,
    total_size: u64,
) -> SendTransferResult {
    let url = format!("http://{}:{}{}", device_ip, device_port, "/api/transfer/request");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id,
        "sender_id": sender_ip,
        "sender_name": sender_name,
        "sender_ip": sender_ip,
        "sender_port": sender_port,
        "files": files,
        "total_size": total_size
    });

    println!("发送传输请求到: {}", url);
    println!("请求内容: {}", body);

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            return SendTransferResult {
                success: false,
                message: format!("发送请求失败: {}", e),
            };
        }
    };

    println!("响应状态: {}", response.status());

    if response.status().is_success() {
        SendTransferResult {
            success: true,
            message: "请求已发送".to_string(),
        }
    } else {
        SendTransferResult {
            success: false,
            message: format!("请求失败: {}", response.status()),
        }
    }
}

// 发送文本请求到目标设备
#[command]
async fn send_text_request(
    device_ip: String,
    device_port: u16,
    transfer_id: String,
    sender_name: String,
    sender_ip: String,
    sender_port: u16,
    text_content: String,
) -> SendTransferResult {
    let url = format!("http://{}:{}{}", device_ip, device_port, "/api/transfer/request");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id,
        "sender_id": sender_ip,
        "sender_name": sender_name,
        "sender_ip": sender_ip,
        "sender_port": sender_port,
        "files": [],
        "total_size": text_content.chars().count() as u64,
        "request_type": "text",
        "text_content": text_content
    });

    println!("发送文本请求到: {}", url);

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            return SendTransferResult {
                success: false,
                message: format!("发送文本失败: {}", e),
            };
        }
    };

    if response.status().is_success() {
        SendTransferResult {
            success: true,
            message: "文本已发送".to_string(),
        }
    } else {
        SendTransferResult {
            success: false,
            message: format!("发送失败: {}", response.status()),
        }
    }
}

// 发送接受传输请求到发送方
#[command]
async fn send_accept_request(
    sender_ip: String,
    sender_port: u16,
    transfer_id: String,
    save_path: String,
    receiver_ip: String,
    receiver_port: u16,
) -> SendTransferResult {
    let url = format!("http://{}:{}{}", sender_ip, sender_port, "/api/transfer/accept");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id,
        "save_path": save_path,
        "receiver_ip": receiver_ip,
        "receiver_port": receiver_port
    });

    println!("发送接受请求到: {}", url);

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            return SendTransferResult {
                success: false,
                message: format!("发送接受请求失败: {}", e),
            };
        }
    };

    if response.status().is_success() {
        SendTransferResult {
            success: true,
            message: "接受成功".to_string(),
        }
    } else {
        SendTransferResult {
            success: false,
            message: format!("接受失败: {}", response.status()),
        }
    }
}

// 发送拒绝传输请求到发送方
#[command]
async fn send_reject_request(
    sender_ip: String,
    sender_port: u16,
    transfer_id: String,
) -> SendTransferResult {
    let url = format!("http://{}:{}{}", sender_ip, sender_port, "/api/transfer/reject");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id
    });

    println!("发送拒绝请求到: {}", url);

    let _response = match client.post(&url).json(&body).send().await {
        Ok(_) => {},
        Err(_) => {}
    };

    SendTransferResult {
        success: true,
        message: "已拒绝".to_string(),
    }
}

// 本地存储传输保存路径（接收方使用）
#[command]
async fn save_transfer_path_locally(
    transfer_id: String,
    save_path: String,
) -> SendTransferResult {
    let http_port = transfer::get_http_port();
    let url = format!("http://127.0.0.1:{}{}", http_port, "/api/transfer/save-path");
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "transfer_id": transfer_id,
        "save_path": save_path
    });

    println!("本地存储保存路径: {}", url);

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            return SendTransferResult {
                success: false,
                message: format!("存储保存路径失败: {}", e),
            };
        }
    };

    if response.status().is_success() {
        SendTransferResult {
            success: true,
            message: "保存路径已存储".to_string(),
        }
    } else {
        SendTransferResult {
            success: false,
            message: format!("存储失败: {}", response.status()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_local_ip,
            get_http_port,
            upload_file_to_receiver,
            query_upload_status,
            get_transfer_history,
            save_transfer_record,
            update_transfer_status,
            save_file_record,
            get_transfer_files,
            list_folder_files,
            get_file_size,
            calculate_file_md5,
            send_transfer_request,
            send_text_request,
            send_accept_request,
            send_reject_request,
            save_transfer_path_locally,
            get_settings,
            set_default_save_path,
            validate_save_path,
            trigger_device_scan
        ])
        .setup(|app| {
            // 启动设备发现服务
            let app_handle = app.handle();
            discovery::start_discovery(app_handle.clone());

            // 启动HTTP传输服务
            transfer::start_http_server(app_handle.clone());

            // 初始化历史记录数据库
            history::init_database(app_handle.clone());

            // 初始化设置
            settings::init_settings(app_handle.clone());

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // 应用退出时发送bye消息通知其他设备
                discovery::stop_discovery();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}