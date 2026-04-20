pub mod broadcaster;
pub mod listener;
pub mod device_registry;

use std::sync::Arc;
use tauri::AppHandle;
use uuid::Uuid;
use device_registry::DeviceRegistry;

/// 默认UDP发现端口，可通过环境变量 LAN_DISCOVERY_PORT 覆盖
pub fn get_discovery_port() -> u16 {
    std::env::var("LAN_DISCOVERY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3737)
}

/// 设备信息结构
#[derive(Clone)]
pub struct LocalDeviceInfo {
    pub device_id: String,
    pub device_name: String,
}

/// 存储本机设备信息的全局变量（用于退出时发送bye）
static mut LOCAL_DEVICE_INFO: Option<LocalDeviceInfo> = None;

/// 启动设备发现服务
pub fn start_discovery(app_handle: AppHandle) {
    let app_handle = Arc::new(app_handle);
    let discovery_port = get_discovery_port();

    // 生成本机设备信息（广播和监听共用）
    let device_info = LocalDeviceInfo {
        device_id: Uuid::new_v4().to_string(),
        device_name: gethostname::gethostname()
            .to_string_lossy()
            .to_string(),
    };

    // 存储设备信息用于退出时发送bye
    unsafe {
        LOCAL_DEVICE_INFO = Some(device_info.clone());
    }

    // 创建设备注册表（用于管理设备超时离线）
    let registry = Arc::new(DeviceRegistry::new((*app_handle).clone()));
    registry.clone().start_cleanup_task();

    // 启动UDP广播发送
    broadcaster::start_broadcaster(app_handle.clone(), device_info.clone(), discovery_port);

    // 启动UDP监听（传入设备注册表）
    listener::start_listener(app_handle.clone(), device_info.clone(), registry, discovery_port);
}

/// 停止设备发现服务并发送bye消息
#[allow(static_mut_refs)]
pub fn stop_discovery() {
    unsafe {
        if let Some(info) = LOCAL_DEVICE_INFO.take() {
            let discovery_port = get_discovery_port();
            broadcaster::stop_broadcaster(&info, discovery_port);
        }
    }
}

/// 获取本机局域网IP
pub fn get_local_ip() -> String {
    broadcaster::get_local_ip()
}

/// 手动触发设备扫描
pub fn trigger_scan() {
    broadcaster::send_announce_now();
}