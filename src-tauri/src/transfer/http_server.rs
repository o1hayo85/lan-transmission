use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::{Path, State},
    http::Method,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Any};
use super::get_http_port;
use super::upload_handler;
use super::download_handler;

/// 传输请求
#[derive(Deserialize, Serialize, Clone)]
pub struct TransferRequest {
    pub transfer_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub sender_ip: String,
    pub sender_port: u16,
    pub files: Vec<FileInfo>,
    pub total_size: u64,
    /// 请求类型: "file" 或 "text"
    #[serde(default = "default_request_type")]
    pub request_type: String,
    /// 文本内容（当 request_type 为 "text" 时）
    #[serde(default)]
    pub text_content: Option<String>,
}

fn default_request_type() -> String {
    "file".to_string()
}

/// 文件信息
#[derive(Deserialize, Serialize, Clone)]
pub struct FileInfo {
    pub file_id: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub relative_path: Option<String>,
}

/// 应用状态
pub struct AppState {
    pub app_handle: AppHandle,
    pub pending_transfers: Mutex<Vec<TransferRequest>>,
    /// 存储每个传输的保存路径 (transfer_id -> save_path)
    pub save_paths: Mutex<HashMap<String, String>>,
    /// 存储发送方待下载的文件路径 (file_id -> file_path)
    pub download_files: Mutex<HashMap<String, String>>,
    /// 存储已取消的传输ID
    pub cancelled_transfers: Mutex<Vec<String>>,
}

/// 启动HTTP服务器
pub fn start(app_handle: AppHandle) {
    let http_port = get_http_port();
    let state = Arc::new(AppState {
        app_handle,
        pending_transfers: Mutex::new(Vec::new()),
        save_paths: Mutex::new(HashMap::new()),
        download_files: Mutex::new(HashMap::new()),
        cancelled_transfers: Mutex::new(Vec::new()),
    });

    // 配置CORS，允许跨域请求
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    let router = Router::new()
        .route("/api/transfer/request", post(handle_transfer_request))
        .route("/api/transfer/accept", post(handle_accept))
        .route("/api/transfer/reject", post(handle_reject))
        .route("/api/transfer/save-path", post(handle_save_path))
        .route("/api/upload", post(upload_handler::handle_upload))
        .route("/api/upload/status", post(handle_upload_status))
        .route("/api/download/:file_id", get(download_handler::handle_download))
        .route("/api/status/:transfer_id", get(handle_status))
        .route("/api/transfer/cancel", post(handle_cancel))
        .layer(cors)
        .with_state(state);

    std::thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", http_port);
        println!("HTTP服务器启动，端口: {}", http_port);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    });
}

/// 处理传输请求
async fn handle_transfer_request(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TransferRequest>,
) -> Json<serde_json::Value> {
    println!("收到传输请求: transfer_id={}, sender={}, type={}, files={}",
        request.transfer_id, request.sender_name, request.request_type, request.files.len());

    // 根据请求类型发送不同事件
    let event_name = if request.request_type == "text" {
        "text-request"
    } else {
        // 存储待处理的传输请求（仅文件传输）
        state.pending_transfers.lock().await.push(request.clone());
        "transfer-request"
    };

    // 发送事件到前端
    if let Some(window) = state.app_handle.get_webview_window("main") {
        println!("找到main window，发送 {} 事件", event_name);
        let emit_result = window.emit(event_name, &request);
        println!("事件发送结果: {:?}", emit_result);
    } else {
        println!("未找到main window，尝试通过AppHandle发送");
        let emit_result = state.app_handle.emit(event_name, &request);
        println!("AppHandle发送结果: {:?}", emit_result);
    }

    Json(serde_json::json!({
        "success": true,
        "message": "请求已发送，等待接收方确认"
    }))
}

/// 处理接受传输（发送方服务器接收此请求）
async fn handle_accept(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");
    let receiver_save_path = body["save_path"].as_str().unwrap_or("");
    let receiver_ip = body["receiver_ip"].as_str().unwrap_or("");

    // 详细日志：打印接收到的完整body
    println!("收到accept请求完整body: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

    // 解析 receiver_port，支持数字和字符串两种格式
    let receiver_port = body["receiver_port"].as_u64()
        .or_else(|| body["receiver_port"].as_str().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(get_http_port() as u64) as u16;

    println!("收到accept请求: transfer_id={}, receiver_ip={}, receiver_port={}", transfer_id, receiver_ip, receiver_port);

    // 从pending_transfers获取传输请求信息
    let transfer_request = {
        let pending = state.pending_transfers.lock().await;
        pending.iter().find(|r| r.transfer_id == transfer_id).cloned()
    };

    // 保存接收方选择的保存路径
    if !receiver_save_path.is_empty() {
        state.save_paths.lock().await.insert(transfer_id.to_string(), receiver_save_path.to_string());
    }

    // 通知前端传输已被接受，包含完整信息
    // 需要通过main window发送事件
    if let Some(window) = state.app_handle.get_webview_window("main") {
        println!("找到main window，发送transfer-accepted事件");
        if let Some(request) = transfer_request {
            let emit_result = window.emit("transfer-accepted", serde_json::json!({
                "transfer_id": transfer_id,
                "receiver_ip": receiver_ip,
                "receiver_port": receiver_port,
                "save_path": receiver_save_path,
                "files": request.files,
                "total_size": request.total_size,
                "peer_device_name": request.sender_name
            }));
            println!("transfer-accepted事件发送结果: {:?}", emit_result);
        } else {
            let emit_result = window.emit("transfer-accepted", serde_json::json!({
                "transfer_id": transfer_id,
                "receiver_ip": receiver_ip,
                "receiver_port": receiver_port,
                "save_path": receiver_save_path
            }));
            println!("transfer-accepted事件发送结果(无files): {:?}", emit_result);
        }
    } else {
        println!("未找到main window，无法发送transfer-accepted事件");
    }

    Json(serde_json::json!({
        "success": true,
        "accepted": true
    }))
}

/// 处理拒绝传输
async fn handle_reject(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");

    state.pending_transfers.lock().await.retain(|r| r.transfer_id != transfer_id);

    let _ = state.app_handle.emit("transfer-rejected", transfer_id);

    Json(serde_json::json!({
        "success": true,
        "accepted": false
    }))
}

/// 查询传输状态
async fn handle_status(
    Path(transfer_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "transfer_id": transfer_id,
        "status": "in_progress",
        "transferred_size": 0,
        "total_size": 0
    }))
}

/// 处理取消传输
async fn handle_cancel(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");

    // 将传输添加到取消列表
    state.cancelled_transfers.lock().await.push(transfer_id.to_string());

    // 从待处理列表移除
    state.pending_transfers.lock().await.retain(|r| r.transfer_id != transfer_id);

    // 发送取消事件到前端
    let _ = state.app_handle.emit("transfer-cancelled", transfer_id);

    Json(serde_json::json!({
        "success": true,
        "transfer_id": transfer_id
    }))
}

/// 检查传输是否已取消（供其他handler使用）
pub async fn is_transfer_cancelled(state: &Arc<AppState>, transfer_id: &str) -> bool {
    let cancelled = state.cancelled_transfers.lock().await;
    cancelled.contains(&transfer_id.to_string())
}

/// 查询已上传文件大小（用于断点续传）
async fn handle_upload_status(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");
    let file_name = body["file_name"].as_str().unwrap_or("");
    let relative_path = body["relative_path"].as_str().unwrap_or("");

    // 获取保存路径
    let save_path = {
        let paths = state.save_paths.lock().await;
        paths.get(transfer_id).cloned().unwrap_or_default()
    };

    // 确定文件路径
    let base_path = if save_path.is_empty() {
        let app_dir = state.app_handle.path().app_data_dir().unwrap_or_default();
        app_dir.join("transfers").join(transfer_id)
    } else {
        std::path::PathBuf::from(&save_path)
    };

    let file_path = if relative_path.is_empty() {
        base_path.join(file_name)
    } else {
        base_path.join(relative_path)
    };

    // 检查文件是否存在并获取大小
    let exists = file_path.exists();
    let size = if exists {
        std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    Json(serde_json::json!({
        "success": true,
        "exists": exists,
        "received_size": size
    }))
}

/// 本地存储传输保存路径（接收方使用）
async fn handle_save_path(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");
    let save_path = body["save_path"].as_str().unwrap_or("");

    println!("本地存储保存路径: transfer_id={}, save_path={}", transfer_id, save_path);

    // 存储保存路径
    state.save_paths.lock().await.insert(transfer_id.to_string(), save_path.to_string());

    Json(serde_json::json!({
        "success": true,
        "transfer_id": transfer_id,
        "save_path": save_path
    }))
}