pub mod broadcaster;
pub mod listener;
pub mod device_registry;

use std::sync::Arc;
use tauri::AppHandle;
use uuid::Uuid;

pub const DISCOVERY_PORT: u16 = 3737;

/// 设备信息结构
#[derive(Clone)]
pub struct LocalDeviceInfo {
    pub device_id: String,
    pub device_name: String,
}

/// 启动设备发现服务
pub fn start_discovery(app_handle: AppHandle) {
    let app_handle = Arc::new(app_handle);

    // 生成本机设备信息（广播和监听共用）
    let device_info = LocalDeviceInfo {
        device_id: Uuid::new_v4().to_string(),
        device_name: gethostname::gethostname()
            .to_string_lossy()
            .to_string(),
    };

    // 启动UDP广播发送
    broadcaster::start_broadcaster(app_handle.clone(), device_info.clone());

    // 启动UDP监听
    listener::start_listener(app_handle.clone(), device_info.clone());
}

/// 获取本机局域网IP
pub fn get_local_ip() -> String {
    broadcaster::get_local_ip()
}