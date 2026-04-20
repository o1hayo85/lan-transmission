use std::net::{UdpSocket, SocketAddr, Ipv4Addr};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use super::broadcaster::DiscoveryMessage;
use super::{DISCOVERY_PORT, LocalDeviceInfo};
use super::broadcaster::MULTICAST_ADDR;

/// 启动UDP监听服务（监听多播地址）
pub fn start_listener(app_handle: Arc<AppHandle>, device_info: LocalDeviceInfo) {
    std::thread::spawn(move || {
        // 绑定到0.0.0.0接收多播
        let bind_addr = format!("0.0.0.0:{}", DISCOVERY_PORT);
        let socket = UdpSocket::bind(&bind_addr).expect(&format!("无法绑定到 {}", bind_addr));

        // 获取真实局域网接口IP并加入多播组
        let lan_ip = super::broadcaster::get_local_ip();
        let lan_ip_parsed: Ipv4Addr = lan_ip.parse().expect("无效的IP地址");

        // 加入多播组
        socket.join_multicast_v4(MULTICAST_ADDR, lan_ip_parsed)
            .expect("无法加入多播组");

        println!("监听服务绑定到: {}, 加入多播组: {}", bind_addr, MULTICAST_ADDR);
        println!("监听服务启动，端口: {}, 本机ID: {}", DISCOVERY_PORT, device_info.device_id);

        let mut buf = [0u8; 1024];

        loop {
            match socket.recv_from(&mut buf) {
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

                            // 发送事件到前端
                            let _ = app_handle.emit("device-discovered", &msg);

                            // 如果收到announce，回复本机信息
                            if msg.msg_type == "announce" {
                                let local_ip = super::broadcaster::get_local_ip();
                                let response = DiscoveryMessage {
                                    msg_type: "response".to_string(),
                                    device_id: device_info.device_id.clone(),
                                    device_name: device_info.device_name.clone(),
                                    ip: local_ip,
                                    port: 8080,
                                    timestamp: chrono::Local::now().timestamp(),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                // 回复到发送方地址
                                println!("回复response: {} -> {}", json, addr);
                                let _ = socket.send_to(json.as_bytes(), addr);
                            }
                        } else if msg.msg_type == "bye" {
                            println!("设备离线: {}", msg.device_id);
                            let _ = app_handle.emit("device-lost", &msg.device_id);
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