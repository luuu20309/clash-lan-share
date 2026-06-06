<div align="center">

# 🌿 Clash LAN Share

**让局域网设备共享 Clash 代理连接**

[![Release](https://img.shields.io/github/v/release/luuu/clash-lan-share?color=7b9e87&style=flat-square)](https://github.com/luuu/clash-lan-share/releases)
[![License](https://img.shields.io/badge/license-MIT-7b9e87?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows-7b9e87?style=flat-square)]()

一个轻量、美观的桌面工具，让你轻松将 Clash 代理共享给局域网内的其他设备。

![demo](./docs/demo.png)

</div>

## ✨ 特性

- 🔍 **自动检测** - 支持 Clash / Clash Verge / Clash Verge Rev / mihomo
- 🌐 **一键共享** - 快速开启/关闭局域网代理共享
- 📡 **代理信息** - 显示 HTTP 和 SOCKS5 代理地址，支持一键复制
- 📱 **设备扫描** - 扫描局域网中已连接的设备
- ⚡ **模式切换** - 快速切换规则/全局/直连模式
- 📋 **运行日志** - 实时显示操作日志
- 🎨 **精美界面** - 纸质质感 + 淡雅绿色风格，HarmonyOS Sans 字体
- 🪶 **轻量小巧** - 安装包仅 3MB，内存占用低

## 📦 下载

前往 [Releases](https://github.com/luuu/clash-lan-share/releases) 页面下载最新版本。

| 文件 | 说明 |
|------|------|
| `Clash LAN Share_x.x.x_x64-setup.exe` | NSIS 安装包（推荐） |
| `Clash LAN Share_x.x.x_x64_en-US.msi` | MSI 安装包 |

## 🚀 使用方法

### 前置条件

1. 已安装并运行 [Clash](https://github.com/Dreamacro/clash) / [Clash Verge](https://github.com/zzzgydi/clash-verge) / [Clash Verge Rev](https://github.com/clash-verge-rev/clash-verge-rev) / [mihomo](https://github.com/MetaCubeX/mihomo)
2. Clash 已开启 RESTful API（默认已开启）

### 操作步骤

1. **运行本工具** - 双击安装或运行 exe
2. **自动检测** - 工具会自动检测正在运行的 Clash
3. **开启共享** - 点击「开启局域网共享」按钮
4. **配置设备** - 在其他设备上设置 HTTP 代理

### 其他设备代理设置

<details>
<summary>Windows</summary>

设置 → 网络和 Internet → 代理 → 手动设置代理

地址：`你的IP`，端口：`7890`（或你的代理端口）
</details>

<details>
<summary>macOS</summary>

系统偏好设置 → 网络 → 高级 → 代理 → Web 代理(HTTP)

服务器：`你的IP`，端口：`7890`
</details>

<details>
<summary>iOS</summary>

设置 → Wi-Fi → 点击已连接的网络 → 配置代理 → 手动

服务器：`你的IP`，端口：`7890`
</details>

<details>
<summary>Android</summary>

设置 → WLAN → 长按已连接的网络 → 修改网络 → 高级选项 → 代理 → 手动

主机名：`你的IP`，端口：`7890`
</details>

<details>
<summary>Linux / 终端</summary>

```bash
export http_proxy=http://你的IP:7890
export https_proxy=http://你的IP:7890
```
</details>

## 🛠️ 开发

### 环境要求

- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/)（可选）

### 开发运行

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 克隆项目
git clone https://github.com/luuu/clash-lan-share.git
cd clash-lan-share

# 开发模式
cargo tauri dev

# 构建发布版
cargo tauri build
```

### 项目结构

```
clash-lan-share/
├── src/                    # 前端资源
│   ├── index.html          # 主页面
│   ├── style.css           # 样式（纸质纹理 + 绿色主题）
│   └── app.js              # 前端逻辑
├── src-tauri/              # Rust 后端
│   ├── Cargo.toml          # 依赖配置
│   ├── tauri.conf.json     # Tauri 配置
│   ├── capabilities/       # 权限配置
│   └── src/
│       ├── main.rs         # 入口
│       ├── lib.rs          # Tauri 命令定义
│       ├── clash.rs        # Clash API 管理
│       └── lan.rs          # 局域网管理
└── README.md
```

## 🎨 设计

- **字体**: HarmonyOS Sans
- **配色**: 淡雅绿色系（`#7b9e87` 主色）
- **风格**: 纸质纹理 + 圆角卡片 + 柔和阴影
- **窗口**: 无边框圆角窗口 + 自定义标题栏

## 📄 License

[MIT](LICENSE)

---

<div align="center">

**如果觉得有用，请给个 ⭐ Star 支持一下喵～**

</div>
