use std::net::{UdpSocket, SocketAddr, Ipv4Addr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use tauri::AppHandle;
use super::LocalDeviceInfo;
use crate::transfer::get_http_port;

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
#[derive(Clone)]
pub struct LanInterface {
    pub ip: String,
    #[allow(dead_code)]
    interface_name: String,
}

/// 获取真实的局域网接口（排除代理/VPN/虚拟接口）
pub fn get_lan_interface() -> Option<LanInterface> {
    let interfaces = if_addrs::get_if_addrs().unwrap_or_default();

    // 虚拟接口名称关键词（排除这些）
    let virtual_keywords = [
        "vEthernet",      // Hyper-V虚拟交换机
        "WSL",            // Windows Subsystem for Linux
        "Hyper-V",        // Hyper-V
        "Docker",         // Docker Desktop
        "VMware",         // VMware虚拟网卡
        "VirtualBox",     // VirtualBox虚拟网卡
        "vnic",           // 通用虚拟NIC
        "Loopback",       // 回环
        "Tunnel",         // 隧道接口
        "TAP",            // VPN TAP设备
        "TUN",            // VPN TUN设备
        "WireGuard",      // WireGuard VPN
        "vproxy",         // 代理虚拟网卡
        "ZeroTier",       // ZeroTier虚拟网络
        "NordVPN",        // NordVPN
        "Clash",          // Clash代理
        "SingBox",        // SingBox代理
    ];

    // 物理接口名称关键词（优先选择这些）
    let physical_keywords = [
        "Ethernet",
        "Wi-Fi",
        "WLAN",
        "无线",
        "以太网",
        "网络",
        "Realtek",
        "Intel",
        "Broadcom",
        "Qualcomm",
        "MediaTek",
    ];

    // 虚拟网段（排除这些）
    let excluded_prefixes = [
        "198.18.",   // Clash/Shadowsocks等代理软件
        "198.19.",   // 代理软件
        "172.17.",   // Docker默认网段
        "172.18.",   // Docker自定义网段
        "172.19.",   // Docker
        "172.20.",   // Docker/其他虚拟
        "172.21.",   // Docker/其他虚拟
        "172.22.",   // Docker/其他虚拟
        "172.23.",   // Docker/其他虚拟
        "172.24.",   // Hyper-V默认交换机
        "172.25.",   // Hyper-V/WSL
        "172.26.",   // Hyper-V/WSL
        "172.27.",   // Hyper-V/WSL
        "172.28.",   // Hyper-V/WSL
        "172.29.",   // Hyper-V/WSL
        "172.30.",   // Hyper-V/WSL
        "172.31.",   // 其他虚拟网段末尾
        "169.254.",  // APIPA（链路本地地址）
        "100.64.",   // CGNAT（运营商级NAT）
        "192.0.2.",  // TEST-NET-1（测试网络）
        "198.51.100.", // TEST-NET-2
        "203.0.113.", // TEST-NET-3
        "224.",      // 多播地址范围
        "239.",      // 多播地址范围
    ];

    // 按优先级筛选接口：物理接口优先
    let mut physical_candidates: Vec<LanInterface> = Vec::new();
    let mut other_candidates: Vec<LanInterface> = Vec::new();

    for iface in interfaces.clone() {
        let ip = iface.addr.ip();
        if !ip.is_ipv4() || ip.is_loopback() {
            continue;
        }

        let ip_str = ip.to_string();
        let name_lower = iface.name.to_lowercase();

        // 检查是否是虚拟接口名称
        let is_virtual_name = virtual_keywords.iter().any(|k| name_lower.contains(&k.to_lowercase()));
        if is_virtual_name {
            println!("跳过虚拟接口: {} ({}) - 名称匹配", ip_str, iface.name);
            continue;
        }

        // 检查是否在排除的虚拟网段
        let is_excluded_ip = excluded_prefixes.iter().any(|p| ip_str.starts_with(p));
        if is_excluded_ip {
            println!("跳过虚拟网段: {} ({}) - IP匹配", ip_str, iface.name);
            continue;
        }

        // 检查是否是常见局域网网段
        let is_private_range = ip_str.starts_with("192.168.")
            || ip_str.starts_with("10.")
            || (ip_str.starts_with("172.") && is_private_172(&ip_str));

        if !is_private_range {
            println!("跳过非私有网段: {} ({})", ip_str, iface.name);
            continue;
        }

        // 检查是否是物理接口名称
        let is_physical_name = physical_keywords.iter().any(|k| name_lower.contains(&k.to_lowercase()));

        let candidate = LanInterface {
            ip: ip_str.clone(),
            interface_name: iface.name.clone(),
        };

        if is_physical_name {
            println!("候选物理接口: {} ({}) - 名称匹配物理设备", ip_str, iface.name);
            physical_candidates.push(candidate);
        } else {
            println!("候选接口: {} ({})", ip_str, iface.name);
            other_candidates.push(candidate);
        }
    }

    // 优先返回物理接口
    if let Some(best) = physical_candidates.first() {
        println!("选择物理接口: {} ({})", best.ip, best.interface_name);
        return Some(best.clone());
    }

    // 其次返回其他候选
    if let Some(best) = other_candidates.first() {
        println!("选择接口: {} ({})", best.ip, best.interface_name);
        return Some(best.clone());
    }

    // 最后尝试通过连接外网获取本地IP
    println!("未找到局域网接口，尝试通过外网连接检测...");
    let fallback_ip = get_ip_via_connection();
    if let Some(ip) = fallback_ip {
        println!("通过外网连接检测到IP: {}", ip);
        return Some(LanInterface {
            ip,
            interface_name: "auto-detected".to_string(),
        });
    }

    None
}

/// 通过连接外网获取本地IP（绕过虚拟接口）
fn get_ip_via_connection() -> Option<String> {
    // 尝试连接多个公共DNS服务器，获取本地IP
    let targets = ["8.8.8.8:80", "1.1.1.1:80", "208.67.222.222:80"];

    for target in targets {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(target).is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip().to_string();
                    // 确保不是虚拟网段
                    let ip_str = ip.clone();
                    if ip_str.starts_with("192.168.") || ip_str.starts_with("10.") {
                        return Some(ip);
                    }
                }
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

/// 全局停止信号
static STOP_SIGNAL: AtomicBool = AtomicBool::new(false);

/// 全局存储设备信息用于手动扫描
static mut DEVICE_INFO_FOR_SCAN: Option<LocalDeviceInfo> = None;

/// 立即发送一次announce消息（用于手动触发扫描）
pub fn send_announce_now() {
    // 使用指针安全访问 static mut
    let device_info_ptr = std::ptr::addr_of!(DEVICE_INFO_FOR_SCAN);
    unsafe {
        if let Some(device_info) = (*device_info_ptr).as_ref() {
            let lan_interface = get_lan_interface();
            if let Some(lan) = lan_interface {
                let http_port = get_http_port();
                let bind_addr = format!("{}:0", lan.ip);
                if let Ok(socket) = UdpSocket::bind(&bind_addr) {
                    socket.set_multicast_ttl_v4(1).ok();
                    let multicast_addr: SocketAddr = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_TARGET_PORT);

                    let msg = DiscoveryMessage {
                        msg_type: "announce".to_string(),
                        device_id: device_info.device_id.clone(),
                        device_name: device_info.device_name.clone(),
                        ip: lan.ip.clone(),
                        port: http_port,
                        timestamp: chrono::Local::now().timestamp(),
                    };
                    let json = serde_json::to_string(&msg).unwrap();
                    println!("手动触发扫描: {}", json);
                    socket.send_to(json.as_bytes(), multicast_addr).ok();
                }
            }
        }
    }
}

/// 停止广播并发送bye消息
pub fn stop_broadcaster(device_info: &LocalDeviceInfo, _discovery_port: u16) {
    STOP_SIGNAL.store(true, Ordering::SeqCst);

    // 发送bye消息通知其他设备
    let lan_interface = get_lan_interface();
    let http_port = get_http_port();
    if let Some(lan) = lan_interface {
        let bind_addr = format!("{}:0", lan.ip);
        if let Ok(socket) = UdpSocket::bind(&bind_addr) {
            socket.set_multicast_ttl_v4(1).ok();
            // 发送到固定的多播目标端口
            let multicast_addr: SocketAddr = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_TARGET_PORT);

            let msg = DiscoveryMessage {
                msg_type: "bye".to_string(),
                device_id: device_info.device_id.clone(),
                device_name: device_info.device_name.clone(),
                ip: lan.ip.clone(),
                port: http_port,
                timestamp: chrono::Local::now().timestamp(),
            };
            let json = serde_json::to_string(&msg).unwrap();
            println!("发送bye消息: {}", json);
            socket.send_to(json.as_bytes(), multicast_addr).ok();
        }
    }
}

/// 多播目标端口（固定，所有实例都发送到这个端口）
pub const MULTICAST_TARGET_PORT: u16 = 3737;

/// 启动UDP多播发送
pub fn start_broadcaster(_app_handle: Arc<AppHandle>, device_info: LocalDeviceInfo, _discovery_port: u16) {
    // 存储设备信息用于手动扫描
    unsafe {
        DEVICE_INFO_FOR_SCAN = Some(device_info.clone());
    }

    std::thread::spawn(move || {
        let lan_interface = get_lan_interface();
        if lan_interface.is_none() {
            eprintln!("未找到真实局域网接口");
            return;
        }
        let lan = lan_interface.unwrap();
        let http_port = get_http_port();

        // 绑定到真实网卡IP
        let bind_addr = format!("{}:0", lan.ip);
        let socket = UdpSocket::bind(&bind_addr).expect(&format!("无法绑定到 {}", bind_addr));

        // 设置多播TTL
        socket.set_multicast_ttl_v4(1).expect("无法设置多播TTL");

        // 所有实例都发送到固定的多播目标端口（3737）
        let multicast_addr: SocketAddr = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_TARGET_PORT);

        println!("多播广播服务启动，绑定到: {}, 设备ID: {}, HTTP端口: {}, 目标端口: {}", bind_addr, device_info.device_id, http_port, MULTICAST_TARGET_PORT);

        loop {
            // 检查停止信号
            if STOP_SIGNAL.load(Ordering::SeqCst) {
                println!("广播服务停止");
                break;
            }

            let msg = DiscoveryMessage {
                msg_type: "announce".to_string(),
                device_id: device_info.device_id.clone(),
                device_name: device_info.device_name.clone(),
                ip: lan.ip.clone(),
                port: http_port,
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