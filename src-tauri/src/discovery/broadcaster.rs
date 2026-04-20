use std::net::{UdpSocket, SocketAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tauri::AppHandle;
use super::{DISCOVERY_PORT, LocalDeviceInfo};

/// 多播地址（239.x.x.x 是组织本地范围，不会被代理拦截）
pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// 发现消息结构
#[derive(Serialize, Deserialize, Clone)]
pub struct DiscoveryMessage {
    pub msg_type: String,      // "announce" | "response" | "bye"
    pub device_id: String,
    pub device_name: String,
    pub ip: String,
    pub port: u16,
    pub timestamp: i64,
}

/// 局域网接口信息
pub struct LanInterface {
    pub ip: String,
    pub interface_name: String,
}

/// 获取真实的局域网接口（排除代理/VPN接口）
pub fn get_lan_interface() -> Option<LanInterface> {
    let interfaces = if_addrs::get_if_addrs().unwrap_or_default();

    // 排除代理软件常用的虚拟网段
    let excluded_prefixes = [
        "198.18.",   // Clash/Shadowsocks 等代理软件
        "198.19.",
        "172.19.",   // Docker
        "169.254.",  // APIPA
        "100.64.",   // CGNAT
    ];

    for iface in interfaces {
        let ip = iface.addr.ip();
        if ip.is_ipv4() && !ip.is_loopback() {
            let ip_str = ip.to_string();

            // 排除虚拟/代理网段
            let is_excluded = excluded_prefixes.iter().any(|p| ip_str.starts_with(p));
            if is_excluded {
                continue;
            }

            // 只选择常见局域网网段
            if ip_str.starts_with("192.168.")
               || ip_str.starts_with("10.")
               || (ip_str.starts_with("172.") && is_private_172(&ip_str)) {

                println!("找到局域网接口: {} ({})", ip_str, iface.name);

                return Some(LanInterface {
                    ip: ip_str,
                    interface_name: iface.name,
                });
            }
        }
    }

    None
}

/// 判断172.x.x.x是否是私有网段
fn is_private_172(ip_str: &str) -> bool {
    let parts: Vec<&str> = ip_str.split('.').collect();
    if parts.len() >= 2 {
        let second: u8 = parts[1].parse().unwrap_or(0);
        return second >= 16 && second <= 31;
    }
    false
}

/// 获取本机局域网IP
pub fn get_local_ip() -> String {
    get_lan_interface().map(|i| i.ip).unwrap_or_else(|| {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect("8.8.8.8:80").unwrap();
        let local_addr = socket.local_addr().unwrap();
        local_addr.ip().to_string()
    })
}

/// 启动UDP多播发送
pub fn start_broadcaster(_app_handle: Arc<AppHandle>, device_info: LocalDeviceInfo) {
    std::thread::spawn(move || {
        let lan_interface = get_lan_interface();
        if lan_interface.is_none() {
            eprintln!("未找到真实局域网接口");
            return;
        }
        let lan = lan_interface.unwrap();

        // 绑定到真实网卡IP
        let bind_addr = format!("{}:0", lan.ip);
        let socket = UdpSocket::bind(&bind_addr).expect(&format!("无法绑定到 {}", bind_addr));

        // 设置多播TTL
        socket.set_multicast_ttl_v4(1).expect("无法设置多播TTL");

        let multicast_addr: SocketAddr = SocketAddr::new(MULTICAST_ADDR.into(), DISCOVERY_PORT);

        println!("多播广播服务启动，绑定到: {}, 设备ID: {}", bind_addr, device_info.device_id);

        loop {
            let msg = DiscoveryMessage {
                msg_type: "announce".to_string(),
                device_id: device_info.device_id.clone(),
                device_name: device_info.device_name.clone(),
                ip: lan.ip.clone(),
                port: 8080,
                timestamp: chrono::Local::now().timestamp(),
            };
            let json = serde_json::to_string(&msg).unwrap();

            println!("发送多播: {} -> {}", json, multicast_addr);

            if let Err(e) = socket.send_to(json.as_bytes(), multicast_addr) {
                eprintln!("多播发送失败: {}", e);
            }

            std::thread::sleep(Duration::from_secs(5));
        }
    });
}