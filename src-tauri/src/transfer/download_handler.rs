use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
    body::Body,
};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::http_server::AppState;

/// 处理文件下载（支持断点续传）
pub async fn handle_download(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    // 获取文件路径
    let file_path = {
        let files = state.download_files.lock().await;
        files.get(&file_id).cloned()
    };

    let file_path = match file_path {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("文件未找到"))
                .unwrap();
        }
    };

    // 打开文件
    let mut file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("无法打开文件"))
                .unwrap();
        }
    };

    // 获取文件大小
    let file_size = match file.metadata().await {
        Ok(m) => m.len(),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("无法获取文件信息"))
                .unwrap();
        }
    };

    // 检查 Range 请求头（断点续传）
    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    if let Some(range) = range_header {
        // 解析 Range: bytes=start-end
        let range = range.strip_prefix("bytes=").unwrap_or(range);
        let parts: Vec<&str> = range.split('-').collect();

        let start: u64 = parts[0].parse().unwrap_or(0);
        let end: u64 = if parts.len() > 1 && !parts[1].is_empty() {
            parts[1].parse().unwrap_or(file_size - 1)
        } else {
            file_size - 1
        };

        // 定位到起始位置
        file.seek(std::io::SeekFrom::Start(start)).await.unwrap();

        // 读取指定范围
        let length = end - start + 1;
        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer).await.unwrap();

        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size))
            .header(header::CONTENT_LENGTH, length)
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from(buffer))
            .unwrap()
    } else {
        // 完整文件下载
        let mut buffer = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut buffer).await.unwrap();

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, file_size)
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from(buffer))
            .unwrap()
    }
}