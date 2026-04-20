use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(15),
        }
    }

    /// 注册或更新设备
    pub fn register(&self, device: Device) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.id.clone(), device);
    }

    /// 移除设备
    pub fn remove(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        devices.remove(device_id);
    }

    /// 获取在线设备列表
    pub fn get_online_devices(&self) -> Vec<Device> {
        let mut devices = self.devices.lock().unwrap();
        let now = Instant::now();

        // 移除超时设备
        devices.retain(|_, d| now.duration_since(d.last_seen) < self.timeout);

        devices.values().cloned().collect()
    }

    /// 检查设备是否在线
    pub fn is_online(&self, device_id: &str) -> bool {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).map_or(false, |d| {
            Instant::now().duration_since(d.last_seen) < self.timeout
        })
    }
}