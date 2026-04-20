use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri::Emitter;

/// 设备信息
#[derive(Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub last_seen: Instant,
}

/// 设备注册表
pub struct DeviceRegistry {
    devices: Arc<Mutex<HashMap<String, Device>>>,
    timeout: Duration,
    app_handle: AppHandle,
}

impl DeviceRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(15),
            app_handle,
        }
    }

    /// 注册或更新设备
    pub fn register(&self, device: Device) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.id.clone(), device);
    }

    /// 移除设备并通知前端
    pub fn remove(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        if devices.remove(device_id).is_some() {
            // 通知前端设备离线
            let _ = self.app_handle.emit("device-lost", device_id);
        }
    }

    /// 获取在线设备列表，同时清理超时设备
    pub fn get_online_devices(&self) -> Vec<Device> {
        let mut devices = self.devices.lock().unwrap();
        let now = Instant::now();

        // 移除超时设备并通知前端
        let offline_ids: Vec<String> = devices
            .iter()
            .filter(|(_, d)| now.duration_since(d.last_seen) >= self.timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for id in offline_ids {
            devices.remove(&id);
            let _ = self.app_handle.emit("device-lost", &id);
        }

        devices.values().cloned().collect()
    }

    /// 检查设备是否在线
    pub fn is_online(&self, device_id: &str) -> bool {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).map_or(false, |d| {
            Instant::now().duration_since(d.last_seen) < self.timeout
        })
    }

    /// 启动定期清理任务
    pub fn start_cleanup_task(self: Arc<Self>) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5));
                // 定期清理超时设备
                self.get_online_devices();
            }
        });
    }
}