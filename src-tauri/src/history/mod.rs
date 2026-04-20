pub mod database;
pub mod models;

use tauri::AppHandle;

/// 初始化数据库
pub fn init_database(app_handle: AppHandle) {
    database::init(app_handle);
}