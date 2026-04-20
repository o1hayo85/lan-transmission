use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use socket2::{Socket, Domain, Type, Protocol};
use super::broadcaster::DiscoveryMessage;
use super::LocalDeviceInfo;
use super::broadcaster::MULTICAST_ADDR;
use super::broadcaster::MULTICAST_TARGET_PORT;
use super::device_registry::{DeviceRegistry, Device};
use crate::transfer::get_http_port;

/// 启动UDP监听服务（监听多播地址）
/// 所有实例都监听固定的多播端口3737，使用SO_REUSEADDR允许多进程绑定
pub fn start_listener(app_handle: Arc<AppHandle>, device_info: LocalDeviceInfo, registry: Arc<DeviceRegistry>, _discovery_port: u16) {
    std::thread::spawn(move || {
        // 使用socket2创建socket并设置SO_REUSEADDR
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .expect("无法创建socket");

        // 设置SO_REUSEADDR允许同一台机器上多个进程监听同一端口
        socket.set_reuse_address(true).expect("无法设置reuse_address");

        // 绑定到固定的多播端口3737
        let bind_addr = SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), MULTICAST_TARGET_PORT);
        socket.bind(&bind_addr.into()).expect(&format!("无法绑定到端口 {}", MULTICAST_TARGET_PORT));

        // 转换为标准UdpSocket
        let udp_socket: UdpSocket = socket.into();

        // 获取真实局域网接口IP并加入多播组
        let lan_ip = super::broadcaster::get_local_ip();
        let lan_ip_parsed: Ipv4Addr = lan_ip.parse().expect("无效的IP地址");

        // 加入多播组
        udp_socket.join_multicast_v4(&MULTICAST_ADDR, &lan_ip_parsed)
            .expect("无法加入多播组");

        println!("监听服务绑定到: 0.0.0.0:{} (SO_REUSEADDR), 加入多播组: {}", MULTICAST_TARGET_PORT, MULTICAST_ADDR);
        println!("监听服务启动，本机ID: {}", device_info.device_id);

        let mut buf = [0u8; 1024];

        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    let data = &buf[..len];
                    println!("收到UDP数据: {} 字节, 来自: {}", len, addr);

                    if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(data) {
                        println!("解析消息: type={}, device_id={}, name={}", msg.msg_type, msg.device_id, msg.device_name);

                        // 忽略自己的消息
                        if msg.device_id == device_info.device_id {
                            println!("忽略自己的消息");
                            continue;
                        }

                        if msg.msg_type == "announce" || msg.msg_type == "response" {
                            println!("发现新设备: {} ({})", msg.device_name, msg.ip);

                            // 注册到设备表（用于超时检测）
                            registry.register(Device {
                                id: msg.device_id.clone(),
                                name: msg.device_name.clone(),
                                ip: msg.ip.clone(),
                                port: msg.port,
                                last_seen: Instant::now(),
                            });

                            // 发送事件到前端
                            let _ = app_handle.emit("device-discovered", &msg);

                            // 如果收到announce，回复本机信息
                            if msg.msg_type == "announce" {
                                let local_ip = super::broadcaster::get_local_ip();
                                let http_port = get_http_port();
                                let response = DiscoveryMessage {
                                    msg_type: "response".to_string(),
                                    device_id: device_info.device_id.clone(),
                                    device_name: device_info.device_name.clone(),
                                    ip: local_ip,
                                    port: http_port,
                                    timestamp: chrono::Local::now().timestamp(),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                // 回复到发送方地址
                                println!("回复response: {} -> {}", json, addr);
                                let _ = udp_socket.send_to(json.as_bytes(), addr);
                            }
                        } else if msg.msg_type == "bye" {
                            println!("设备离线: {}", msg.device_id);
                            // 从注册表移除（会自动发送device-lost事件）
                            registry.remove(&msg.device_id);
                        }
                    } else {
                        println!("无法解析消息: {}", String::from_utf8_lossy(data));
                    }
                }
                Err(e) => {
                    eprintln!("UDP接收错误: {}", e);
                }
            }
        }
    });
}