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
use super::HTTP_PORT;
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
}

/// 启动HTTP服务器
pub fn start(app_handle: AppHandle) {
    let state = Arc::new(AppState {
        app_handle,
        pending_transfers: Mutex::new(Vec::new()),
        save_paths: Mutex::new(HashMap::new()),
        download_files: Mutex::new(HashMap::new()),
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
        .route("/api/upload", post(upload_handler::handle_upload))
        .route("/api/upload/status", post(handle_upload_status))
        .route("/api/download/:file_id", get(download_handler::handle_download))
        .route("/api/status/:transfer_id", get(handle_status))
        .route("/api/transfer/cancel", post(handle_cancel))
        .layer(cors)
        .with_state(state);

    std::thread::spawn(|| {
        let addr = format!("0.0.0.0:{}", HTTP_PORT);
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
    // 存储待处理的传输请求
    state.pending_transfers.lock().await.push(request.clone());

    // 发送事件到前端，显示确认对话框
    let _ = state.app_handle.emit("transfer-request", &request);

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
    let receiver_port = body["receiver_port"].as_u64().unwrap_or(HTTP_PORT as u64) as u16;

    // 通知前端传输已被接受，包含接收方地址信息
    let _ = state.app_handle.emit("transfer-accepted", serde_json::json!({
        "transfer_id": transfer_id,
        "receiver_ip": receiver_ip,
        "receiver_port": receiver_port,
        "save_path": receiver_save_path
    }));

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
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let transfer_id = body["transfer_id"].as_str().unwrap_or("");

    Json(serde_json::json!({
        "success": true,
        "transfer_id": transfer_id
    }))
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