mod clash;
mod lan;

use clash::ClashManager;
use lan::LanManager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 应用状态
pub struct AppState {
    pub clash: Mutex<ClashManager>,
    pub lan: Mutex<LanManager>,
}

/// Clash 状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashStatusInfo {
    pub running: bool,
    pub version: String,
    pub api_port: u16,
    pub mixed_port: u16,
    pub socks_port: u16,
    pub allow_lan: bool,
    pub mode: String,
    pub connection_count: usize,
}

/// 分享信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    pub lan_ip: String,
    pub http_url: String,
    pub socks_url: String,
    pub http_port: u16,
    pub socks_port: u16,
}

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub mac: String,
    pub if_type: String,
}

/// 局域网设备
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanDevice {
    pub ip: String,
    pub mac: String,
}

/// 检测 Clash 状态
#[tauri::command]
async fn detect_clash(state: State<'_, AppState>) -> Result<ClashStatusInfo, String> {
    let mut clash = state.clash.lock().map_err(|e| e.to_string())?;
    clash.detect().map_err(|e| e.to_string())?;
    Ok(clash.status_info())
}

/// 获取当前状态（不重新检测）
#[tauri::command]
async fn get_clash_status(state: State<'_, AppState>) -> Result<ClashStatusInfo, String> {
    let clash = state.clash.lock().map_err(|e| e.to_string())?;
    Ok(clash.status_info())
}

/// 切换局域网共享
#[tauri::command]
async fn toggle_lan(state: State<'_, AppState>, enable: bool) -> Result<bool, String> {
    let mut clash = state.clash.lock().map_err(|e| e.to_string())?;
    clash.set_allow_lan(enable).map_err(|e| e.to_string())
}

/// 设置代理模式
#[tauri::command]
async fn set_mode(state: State<'_, AppState>, mode: String) -> Result<bool, String> {
    let mut clash = state.clash.lock().map_err(|e| e.to_string())?;
    clash.set_mode(&mode).map_err(|e| e.to_string())
}

/// 获取本机局域网 IP
#[tauri::command]
async fn get_lan_ip(state: State<'_, AppState>) -> Result<String, String> {
    let lan = state.lan.lock().map_err(|e| e.to_string())?;
    Ok(lan.get_lan_ip())
}

/// 获取所有网络接口
#[tauri::command]
async fn get_network_interfaces(state: State<'_, AppState>) -> Result<Vec<NetworkInterface>, String> {
    let mut lan = state.lan.lock().map_err(|e| e.to_string())?;
    Ok(lan.get_interfaces())
}

/// 生成分享信息
#[tauri::command]
async fn get_share_info(state: State<'_, AppState>) -> Result<ShareInfo, String> {
    let clash = state.clash.lock().map_err(|e| e.to_string())?;
    let lan = state.lan.lock().map_err(|e| e.to_string())?;
    let ip = lan.get_lan_ip();
    let mixed_port = clash.info.mixed_port;
    // mixed-port 同时支持 HTTP 和 SOCKS5，如果 socks-port 为 0 则使用 mixed-port
    let socks_port = if clash.info.socks_port > 0 {
        clash.info.socks_port
    } else {
        mixed_port
    };
    Ok(ShareInfo {
        lan_ip: ip.clone(),
        http_url: format!("http://{}:{}", ip, mixed_port),
        socks_url: format!("socks5://{}:{}", ip, socks_port),
        http_port: mixed_port,
        socks_port,
    })
}

/// 扫描局域网设备
#[tauri::command]
async fn scan_devices() -> Result<Vec<LanDevice>, String> {
    lan::scan_arp_devices().map_err(|e| e.to_string())
}

/// 设置 API 端口
#[tauri::command]
async fn set_api_port(state: State<'_, AppState>, port: u16) -> Result<(), String> {
    let mut clash = state.clash.lock().map_err(|e| e.to_string())?;
    clash.set_api_port(port);
    Ok(())
}

/// 设置 API Secret
#[tauri::command]
async fn set_api_secret(state: State<'_, AppState>, secret: String) -> Result<(), String> {
    let mut clash = state.clash.lock().map_err(|e| e.to_string())?;
    clash.set_secret(&secret);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            clash: Mutex::new(ClashManager::new()),
            lan: Mutex::new(LanManager::new()),
        })
        .invoke_handler(tauri::generate_handler![
            detect_clash,
            get_clash_status,
            toggle_lan,
            set_mode,
            get_lan_ip,
            get_network_interfaces,
            get_share_info,
            scan_devices,
            set_api_port,
            set_api_secret,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
