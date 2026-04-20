pub mod database;
pub mod models;

use tauri::AppHandle;

/// 初始化设置模块
pub fn init_settings(app_handle: AppHandle) {
    database::init_settings_table(&app_handle);
}