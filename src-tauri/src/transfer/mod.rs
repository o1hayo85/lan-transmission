pub mod http_server;
pub mod upload_handler;
pub mod download_handler;

use tauri::AppHandle;

pub const HTTP_PORT: u16 = 8080;

/// 启动HTTP传输服务器
pub fn start_http_server(app_handle: AppHandle) {
    http_server::start(app_handle);
}