use crate::{LanDevice, NetworkInterface};
use std::net::UdpSocket;
use std::process::Command;

/// 局域网管理器
pub struct LanManager {
    lan_ip: String,
}

impl LanManager {
    pub fn new() -> Self {
        Self {
            lan_ip: String::new(),
        }
    }

    /// 获取本机局域网 IP
    pub fn get_lan_ip(&self) -> String {
        // 方法1：通过 UDP 连接获取
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    let ip = addr.ip().to_string();
                    if !ip.starts_with("127.") {
                        return ip;
                    }
                }
            }
        }

        // 方法2：使用 local_ip_address crate
        if let Ok(ip) = local_ip_address::local_ip() {
            let ip_str = ip.to_string();
            if !ip_str.starts_with("127.") {
                return ip_str;
            }
        }

        // 方法3：通过 ipconfig 命令解析
        self.get_ip_from_ipconfig().unwrap_or_else(|| "127.0.0.1".to_string())
    }

    /// 从 ipconfig 输出解析 IP
    fn get_ip_from_ipconfig(&self) -> Option<String> {
        let output = Command::new("ipconfig").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let line = line.trim();
            if line.contains("IPv4") {
                // 提取 IP 地址
                if let Some(pos) = line.rfind(':') {
                    let ip = line[pos + 1..].trim();
                    if !ip.starts_with("127.") && !ip.starts_with("169.254.") {
                        return Some(ip.to_string());
                    }
                }
            }
        }

        None
    }

    /// 获取所有网络接口
    pub fn get_interfaces(&mut self) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();

        if let Ok(output) = Command::new("ipconfig").arg("/all").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut current_name = String::new();
            let mut current_ip = String::new();
            let mut current_mac = String::new();

            for line in stdout.lines() {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // 检测适配器名称
                if line.contains("适配器") || line.contains("adapter") {
                    // 保存上一个适配器
                    if !current_ip.is_empty()
                        && !current_ip.starts_with("127.")
                        && !current_ip.starts_with("169.254.")
                    {
                        interfaces.push(NetworkInterface {
                            name: current_name.clone(),
                            ip: current_ip.clone(),
                            mac: current_mac.clone(),
                            if_type: guess_type(&current_name),
                        });
                    }
                    current_name = line
                        .trim_end_matches('：')
                        .trim_end_matches(':')
                        .trim()
                        .to_string();
                    current_ip.clear();
                    current_mac.clear();
                } else if line.contains("IPv4") {
                    if let Some(pos) = line.rfind(':') {
                        let ip = line[pos + 1..].trim().to_string();
                        if !ip.starts_with("169.254.") {
                            current_ip = ip;
                        }
                    }
                } else if line.contains("物理地址") || line.contains("Physical Address") {
                    if let Some(pos) = line.rfind(':') {
                        current_mac = line[pos + 1..].trim().to_string();
                    }
                }
            }

            // 最后一个适配器
            if !current_ip.is_empty()
                && !current_ip.starts_with("127.")
                && !current_ip.starts_with("169.254.")
            {
                interfaces.push(NetworkInterface {
                    name: current_name,
                    ip: current_ip,
                    mac: current_mac,
                    if_type: "ethernet".to_string(),
                });
            }
        }

        interfaces
    }
}

/// 猜测网络接口类型
fn guess_type(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("wi-fi") || lower.contains("wifi") || lower.contains("wlan") || lower.contains("wireless") {
        "wifi".to_string()
    } else if lower.contains("loopback") || lower.contains("回环") {
        "loopback".to_string()
    } else if lower.contains("vmware") || lower.contains("virtualbox") || lower.contains("hyper-v") || lower.contains("vethernet") {
        "virtual".to_string()
    } else {
        "ethernet".to_string()
    }
}

/// 扫描 ARP 表获取局域网设备
pub fn scan_arp_devices() -> Result<Vec<LanDevice>, String> {
    let output = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| format!("执行 arp 命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        // 匹配 ARP 表格式: IP 地址 + MAC 地址
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let ip = parts[0];
            let mac = parts[1];

            // 验证 IP 格式
            if ip.parse::<std::net::Ipv4Addr>().is_ok() {
                // 跳过多播和回环地址
                if ip.starts_with("127.")
                    || ip.starts_with("224.")
                    || ip.starts_with("239.")
                    || mac == "(incomplete)"
                {
                    continue;
                }

                // 验证 MAC 格式
                if mac.contains('-') || mac.contains(':') {
                    devices.push(LanDevice {
                        ip: ip.to_string(),
                        mac: mac.to_string(),
                    });
                }
            }
        }
    }

    Ok(devices)
}
