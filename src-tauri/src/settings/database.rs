use rusqlite::{Connection, params};
use tauri::AppHandle;
use super::models::AppSettings;

/// 初始化设置表
pub fn init_settings_table(app_handle: &AppHandle) {
    let conn = crate::history::database::get_connection(app_handle);

    // 创建设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create settings table");

    // 插入默认设置（如果不存在）
    let now = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('default_save_path', '', ?1)",
        params![now],
    ).expect("Failed to insert default settings");
}

/// 获取所有设置
pub fn get_all_settings(conn: &Connection) -> AppSettings {
    let mut settings = AppSettings::default();

    if let Ok(value) = get_setting(conn, "default_save_path") {
        settings.default_save_path = value;
    }

    settings
}

/// 获取单个设置项
pub fn get_setting(conn: &Connection, key: &str) -> Result<String, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
}

/// 设置单个配置项
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    let now = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params![key, value, now],
    )?;
    Ok(())
}

/// 验证路径有效性
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("路径不能为空".to_string());
    }

    let path_buf = std::path::Path::new(path);

    // 检查路径是否存在
    if !path_buf.exists() {
        return Err("路径不存在".to_string());
    }

    // 检查是否是文件夹
    if !path_buf.is_dir() {
        return Err("路径不是文件夹".to_string());
    }

    // 检查写入权限（尝试创建唯一临时文件）
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let test_file = path_buf.join(format!(".write_test_{}", timestamp));
    match std::fs::File::create(&test_file) {
        Ok(_) => {
            std::fs::remove_file(&test_file).ok();
            Ok(())
        }
        Err(_) => Err("没有写入权限".to_string()),
    }
}