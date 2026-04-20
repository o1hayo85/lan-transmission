use serde::{Serialize, Deserialize};

/// 确认请求消息
#[derive(Serialize, Deserialize, Clone)]
pub struct ConfirmRequest {
    pub transfer_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub files: Vec<FileInfo>,
    pub total_size: u64,
    pub timestamp: i64,
}

/// 确认响应消息
#[derive(Serialize, Deserialize, Clone)]
pub struct ConfirmResponse {
    pub transfer_id: String,
    pub accepted: bool,
    pub reason: Option<String>,
}

/// 文件信息
#[derive(Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub file_id: String,
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub relative_path: Option<String>,
}

/// 确认状态
pub enum ConfirmStatus {
    Pending,
    Accepted,
    Rejected,
    Timeout,
}

/// 创建确认请求
pub fn create_confirm_request(
    transfer_id: String,
    sender_id: String,
    sender_name: String,
    files: Vec<FileInfo>,
    total_size: u64,
) -> ConfirmRequest {
    ConfirmRequest {
        transfer_id,
        sender_id,
        sender_name,
        files,
        total_size,
        timestamp: chrono::Local::now().timestamp(),
    }
}

/// 创建确认响应
pub fn create_confirm_response(transfer_id: String, accepted: bool, reason: Option<String>) -> ConfirmResponse {
    ConfirmResponse {
        transfer_id,
        accepted,
        reason,
    }
}