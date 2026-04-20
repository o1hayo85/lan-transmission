/// 传输记录模型
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TransferRecord {
    pub id: String,
    pub direction: String,
    pub status: String,
    pub peer_device_id: String,
    pub peer_device_name: String,
    pub peer_ip: Option<String>,
    pub total_size: u64,
    pub transferred_size: u64,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

/// 文件记录模型
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FileRecord {
    pub id: String,
    pub transfer_id: String,
    pub name: String,
    pub path: Option<String>,
    pub size: u64,
    pub md5: Option<String>,
    pub status: String,
    pub created_at: i64,
}