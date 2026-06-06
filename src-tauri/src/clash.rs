use crate::ClashStatusInfo;
use std::time::Duration;

/// Clash 连接信息
#[derive(Debug, Clone)]
pub struct ClashInfo {
    pub host: String,
    pub port: u16,
    pub secret: String,
    pub mixed_port: u16,
    pub socks_port: u16,
    pub allow_lan: bool,
    pub mode: String,
    pub version: String,
    pub running: bool,
}

impl Default for ClashInfo {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9090,
            secret: String::new(),
            mixed_port: 7890,
            socks_port: 7891,
            allow_lan: false,
            mode: "rule".to_string(),
            version: String::new(),
            running: false,
        }
    }
}

/// Clash 管理器
pub struct ClashManager {
    pub info: ClashInfo,
}

impl ClashManager {
    pub fn new() -> Self {
        Self {
            info: ClashInfo::default(),
        }
    }

    /// API 基础 URL
    fn base_url(&self) -> String {
        format!("http://{}:{}", self.info.host, self.info.port)
    }

    /// GET 请求
    fn api_get(&self, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", self.base_url(), path);
        let mut req = ureq::get(&url).timeout(Duration::from_secs(3));
        if !self.info.secret.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", self.info.secret));
        }
        let resp = req.call().map_err(|e| e.to_string())?;
        let body = resp.into_string().map_err(|e| e.to_string())?;
        serde_json::from_str(&body).map_err(|e| e.to_string())
    }

    /// PATCH 请求
    fn api_patch(&self, path: &str, body: serde_json::Value) -> Result<bool, String> {
        let url = format!("{}{}", self.base_url(), path);
        let mut req = ureq::patch(&url).timeout(Duration::from_secs(3));
        if !self.info.secret.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", self.info.secret));
        }
        req = req.set("Content-Type", "application/json");
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        match req.send_string(&body_str) {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(code, _)) => Ok(code >= 200 && code < 300),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 设置 API 端口
    pub fn set_api_port(&mut self, port: u16) {
        self.info.port = port;
    }

    /// 设置 API Secret
    pub fn set_secret(&mut self, secret: &str) {
        self.info.secret = secret.to_string();
    }

    /// 快速检测端口是否开放
    fn check_port(&self, host: &str, port: u16) -> bool {
        use std::net::TcpStream;
        TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse().unwrap(),
            Duration::from_secs(1),
        )
        .is_ok()
    }

    /// 检测 Clash 并加载配置
    pub fn detect(&mut self) -> Result<(), String> {
        let ports = [7897, 9090, 9097, 9099, 9091, 9092, 9093, 9094, 9095, 7890, 7891, 7892, 7893];

        for &port in &ports {
            self.info.port = port;
            if self.try_connect() {
                return Ok(());
            }
        }

        if let Some(port) = self.read_config_port() {
            self.info.port = port;
            if self.try_connect() {
                return Ok(());
            }
        }

        self.info.running = false;
        Err("未检测到 Clash，请确保 Clash 已启动".to_string())
    }

    /// 尝试连接 Clash API
    fn try_connect(&mut self) -> bool {
        if !self.check_port(&self.info.host, self.info.port) {
            return false;
        }

        match self.api_get("/version") {
            Ok(data) => {
                if let Some(ver) = data.get("version").and_then(|v| v.as_str()) {
                    self.info.version = ver.to_string();
                    self.info.running = true;
                    self.load_config();
                    return true;
                }
            }
            Err(_) => {}
        }

        false
    }

    /// 加载当前配置
    fn load_config(&mut self) {
        if let Ok(data) = self.api_get("/configs") {
            if let Some(mp) = data.get("mixed-port").and_then(|v| v.as_u64()) {
                self.info.mixed_port = mp as u16;
            }
            if let Some(sp) = data.get("socks-port").and_then(|v| v.as_u64()) {
                self.info.socks_port = sp as u16;
            }
            if let Some(al) = data.get("allow-lan").and_then(|v| v.as_bool()) {
                self.info.allow_lan = al;
            }
            if let Some(mode) = data.get("mode").and_then(|v| v.as_str()) {
                self.info.mode = mode.to_string();
            }
        }
    }

    /// 从配置文件读取 API 端口
    fn read_config_port(&self) -> Option<u16> {
        let home = dirs::home_dir()?;
        let appdata = std::env::var("APPDATA").unwrap_or_default();

        let config_paths = [
            format!("{}/io.github.clash-verge-rev.clash-verge-rev", appdata),
            format!("{}/io.github.clash-verge.clash-verge", appdata),
            format!("{}/Clash for Windows", appdata),
            home.join(".config/clash").to_string_lossy().to_string(),
            home.join(".config/mihomo").to_string_lossy().to_string(),
        ];

        for base_dir in &config_paths {
            let config_path = format!("{}/config.yaml", base_dir);
            if let Some(port) = self.parse_config_port(&config_path) {
                return Some(port);
            }
        }

        None
    }

    /// 从 clash config.yaml 解析端口
    fn parse_config_port(&self, path: &str) -> Option<u16> {
        let content = std::fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("external-controller:") {
                let val = line.strip_prefix("external-controller:").unwrap_or("").trim();
                if let Some(port_str) = val.split(':').last() {
                    if let Ok(port) = port_str.trim().parse::<u16>() {
                        if port > 0 {
                            return Some(port);
                        }
                    }
                }
            }
        }
        None
    }

    /// 设置允许局域网连接
    pub fn set_allow_lan(&mut self, allow: bool) -> Result<bool, String> {
        let body = serde_json::json!({"allow-lan": allow});
        let result = self.api_patch("/configs", body)?;
        if result {
            self.info.allow_lan = allow;
        }
        Ok(result)
    }

    /// 设置代理模式
    pub fn set_mode(&mut self, mode: &str) -> Result<bool, String> {
        let body = serde_json::json!({"mode": mode});
        let result = self.api_patch("/configs", body)?;
        if result {
            self.info.mode = mode.to_string();
        }
        Ok(result)
    }

    /// 获取连接数
    pub fn get_connection_count(&self) -> usize {
        if let Ok(data) = self.api_get("/connections") {
            if let Some(conns) = data.get("connections").and_then(|v| v.as_array()) {
                return conns.len();
            }
        }
        0
    }

    /// 获取状态信息
    pub fn status_info(&self) -> ClashStatusInfo {
        let connection_count = if self.info.running {
            self.get_connection_count()
        } else {
            0
        };

        ClashStatusInfo {
            running: self.info.running,
            version: self.info.version.clone(),
            api_port: self.info.port,
            mixed_port: self.info.mixed_port,
            socks_port: self.info.socks_port,
            allow_lan: self.info.allow_lan,
            mode: self.info.mode.clone(),
            connection_count,
        }
    }
}
