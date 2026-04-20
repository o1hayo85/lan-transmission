use rusqlite::{Connection, params};
use tauri::{AppHandle, Manager};

/// 初始化数据库
pub fn init(app_handle: AppHandle) {
    let app_dir = app_handle.path().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();

    let db_path = app_dir.join("history.db");
    let conn = Connection::open(&db_path).unwrap();

    // 创建数据表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ip TEXT NOT NULL,
            last_seen INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transfers (
            id TEXT PRIMARY KEY,
            direction TEXT NOT NULL,
            status TEXT NOT NULL,
            peer_device_id TEXT NOT NULL,
            peer_device_name TEXT NOT NULL,
            total_size INTEGER NOT NULL,
            transferred_size INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            transfer_id TEXT NOT NULL,
            name TEXT NOT NULL,
            path TEXT,
            size INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (transfer_id) REFERENCES transfers(id)
        );"
    ).unwrap();
}

/// 获取数据库连接
pub fn get_connection(app_handle: &AppHandle) -> Connection {
    let app_dir = app_handle.path().app_data_dir().unwrap();
    let db_path = app_dir.join("history.db");
    Connection::open(&db_path).unwrap()
}

/// 添加传输记录
pub fn add_transfer(
    conn: &Connection,
    id: &str,
    direction: &str,
    peer_device_id: &str,
    peer_device_name: &str,
    total_size: u64,
) {
    let now = chrono::Local::now().timestamp();
    conn.execute(
        "INSERT INTO transfers (id, direction, status, peer_device_id, peer_device_name, total_size, transferred_size, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5, 0, ?6)",
        params![id, direction, peer_device_id, peer_device_name, total_size, now],
    ).unwrap();
}

/// 更新传输状态
pub fn update_status(conn: &Connection, id: &str, status: &str, transferred_size: u64) {
    conn.execute(
        "UPDATE transfers SET status = ?1, transferred_size = ?2 WHERE id = ?3",
        params![status, transferred_size, id],
    ).unwrap();
}

/// 获取传输历史
pub fn get_history(conn: &Connection) -> Vec<super::models::TransferRecord> {
    let mut stmt = conn.prepare(
        "SELECT id, direction, status, peer_device_id, peer_device_name, total_size, transferred_size, created_at, completed_at
         FROM transfers ORDER BY created_at DESC LIMIT 100"
    ).unwrap();

    stmt.query_map([], |row| {
        Ok(super::models::TransferRecord {
            id: row.get(0)?,
            direction: row.get(1)?,
            status: row.get(2)?,
            peer_device_id: row.get(3)?,
            peer_device_name: row.get(4)?,
            total_size: row.get(5)?,
            transferred_size: row.get(6)?,
            created_at: row.get(7)?,
            completed_at: row.get(8)?,
        })
    }).unwrap().map(|r| r.unwrap()).collect()
}