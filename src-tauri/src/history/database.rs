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
            peer_ip TEXT,
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
            md5 TEXT,
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
    peer_ip: &str,
    total_size: u64,
) {
    let now = chrono::Local::now().timestamp();
    // 使用 INSERT OR IGNORE 忽略重复ID
    let result = conn.execute(
        "INSERT OR IGNORE INTO transfers (id, direction, status, peer_device_id, peer_device_name, peer_ip, total_size, transferred_size, created_at)
         VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6, 0, ?7)",
        params![id, direction, peer_device_id, peer_device_name, peer_ip, total_size, now],
    );
    if let Err(e) = result {
        println!("添加传输记录失败(忽略): {}", e);
    }
}

/// 添加文件记录
pub fn add_file_record(
    conn: &Connection,
    id: &str,
    transfer_id: &str,
    name: &str,
    path: Option<&str>,
    size: u64,
    md5: Option<&str>,
    status: &str,
) {
    let now = chrono::Local::now().timestamp();
    // 使用 INSERT OR IGNORE 忽略重复ID，忽略外键约束失败
    let result = conn.execute(
        "INSERT OR IGNORE INTO files (id, transfer_id, name, path, size, md5, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, transfer_id, name, path, size, md5, status, now],
    );
    if let Err(e) = result {
        println!("添加文件记录失败(忽略): {}", e);
    }
}

/// 获取传输的文件列表
pub fn get_files_by_transfer(conn: &Connection, transfer_id: &str) -> Vec<super::models::FileRecord> {
    let mut stmt = conn.prepare(
        "SELECT id, transfer_id, name, path, size, md5, status, created_at
         FROM files WHERE transfer_id = ?1"
    ).unwrap();

    stmt.query_map(params![transfer_id], |row| {
        Ok(super::models::FileRecord {
            id: row.get(0)?,
            transfer_id: row.get(1)?,
            name: row.get(2)?,
            path: row.get(3)?,
            size: row.get(4)?,
            md5: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
        })
    }).unwrap().map(|r| r.unwrap()).collect()
}

/// 更新传输状态
pub fn update_status(conn: &Connection, id: &str, status: &str, transferred_size: u64) {
    let now = chrono::Local::now().timestamp();

    // 如果状态为completed，同时更新completed_at
    if status == "completed" {
        let result = conn.execute(
            "UPDATE transfers SET status = ?1, transferred_size = ?2, completed_at = ?3 WHERE id = ?4",
            params![status, transferred_size, now, id],
        );
        if let Err(e) = result {
            println!("更新传输状态失败(忽略): {}", e);
        }
    } else {
        let result = conn.execute(
            "UPDATE transfers SET status = ?1, transferred_size = ?2 WHERE id = ?3",
            params![status, transferred_size, id],
        );
        if let Err(e) = result {
            println!("更新传输状态失败(忽略): {}", e);
        }
    }
}

/// 获取传输历史
pub fn get_history(conn: &Connection) -> Vec<super::models::TransferRecord> {
    let mut stmt = conn.prepare(
        "SELECT id, direction, status, peer_device_id, peer_device_name, peer_ip, total_size, transferred_size, created_at, completed_at
         FROM transfers ORDER BY created_at DESC LIMIT 100"
    ).unwrap();

    stmt.query_map([], |row| {
        Ok(super::models::TransferRecord {
            id: row.get(0)?,
            direction: row.get(1)?,
            status: row.get(2)?,
            peer_device_id: row.get(3)?,
            peer_device_name: row.get(4)?,
            peer_ip: row.get(5)?,
            total_size: row.get(6)?,
            transferred_size: row.get(7)?,
            created_at: row.get(8)?,
            completed_at: row.get(9)?,
        })
    }).unwrap().map(|r| r.unwrap()).collect()
}