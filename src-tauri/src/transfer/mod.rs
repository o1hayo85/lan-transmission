pub mod http_server;
pub mod upload_handler;
pub mod download_handler;

use tauri::AppHandle;

/// 默认HTTP端口，可通过环境变量 LAN_HTTP_PORT 覆盖
pub fn get_http_port() -> u16 {
    std::env::var("LAN_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

/// 启动HTTP传输服务器
pub fn start_http_server(app_handle: AppHandle) {
    http_server::start(app_handle);
}