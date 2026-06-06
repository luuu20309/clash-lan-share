// ========================================
// Clash LAN Share - 前端逻辑
// ========================================

// ============ Tauri 调用封装 ============

// Tauri v2 invoke 封装
async function tauriInvoke(cmd, args) {
  // 优先使用 __TAURI__.core.invoke
  if (window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke(cmd, args);
  }
  // 备选：使用内部 IPC
  if (window.__TAURI_INTERNALS__?.invoke) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
  }
  // 再备选：使用 IPC 直接调用
  if (window.__TAURI_INTERNALS__?.ipc) {
    return new Promise((resolve, reject) => {
      const callbackId = Math.random().toString(36).slice(2);
      window['_' + callbackId] = (result, error) => {
        if (error) reject(error);
        else resolve(result);
        delete window['_' + callbackId];
      };
      window.__TAURI_INTERNALS__.ipc({
        cmd: cmd,
        args: args || {},
        callback: callbackId,
        error: callbackId,
      });
    });
  }
  throw new Error('Tauri API 不可用');
}

// 获取窗口对象
function getWindow() {
  // 方式1: Tauri v2 标准
  if (window.__TAURI__?.window?.getCurrentWindow) {
    return window.__TAURI__.window.getCurrentWindow();
  }
  // 方式2: 直接访问 appWindow
  if (window.__TAURI_WINDOW__) {
    return window.__TAURI_WINDOW__;
  }
  return null;
}

// ============ 状态 ============

let currentStatus = {
  running: false,
  allowLan: false,
  mode: 'rule',
};

// ============ 初始化 ============

async function init() {
  addLog('正在初始化...', 'info');

  // 等待 Tauri API 就绪
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke) {
      ready = true;
      break;
    }
    await new Promise(r => setTimeout(r, 200));
  }

  if (!ready) {
    addLog('✗ Tauri API 加载失败，请重启应用', 'error');
    // 显示调试信息
    const debug = {
      tauri: !!window.__TAURI__,
      internals: !!window.__TAURI_INTERNALS__,
      keys: window.__TAURI__ ? Object.keys(window.__TAURI__) : [],
      iKeys: window.__TAURI_INTERNALS__ ? Object.keys(window.__TAURI_INTERNALS__) : [],
    };
    addLog('调试: ' + JSON.stringify(debug), 'warning');
    return;
  }

  setupWindowControls();
  addLog('✓ 初始化完成', 'success');

  // 检测 Clash
  detectClash();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

// ============ 窗口控制 ============

// 通过 IPC 直接调用窗口操作
async function windowAction(action) {
  // 尝试使用窗口对象
  const win = getWindow();
  if (win) {
    try {
      if (action === 'minimize') return await win.minimize();
      if (action === 'close') return await win.close();
    } catch (e) {
      console.error('Window action failed:', e);
    }
  }

  // 备选：使用 IPC 直接调用
  try {
    if (window.__TAURI_INTERNALS__?.invoke) {
      return await window.__TAURI_INTERNALS__.invoke(`plugin:window|${action}`, {
        label: 'main'
      });
    }
  } catch (e) {
    console.error('IPC action failed:', e);
  }
}

function setupWindowControls() {
  document.getElementById('btnMinimize').addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    windowAction('minimize');
  });

  document.getElementById('btnClose').addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    windowAction('close');
  });
}

// ============ Clash 检测 ============

async function detectClash() {
  setStatus('warning', '正在检测...');
  addLog('正在检测 Clash...', 'info');

  try {
    const info = await tauriInvoke('detect_clash');

    if (info.running) {
      currentStatus.running = true;
      currentStatus.allowLan = info.allow_lan;
      currentStatus.mode = info.mode;

      setStatus('success', '已连接');
      document.getElementById('clashVersion').textContent = info.version;
      document.getElementById('clashPort').textContent = info.api_port;
      document.getElementById('clashMode').textContent = getModeLabel(info.mode);
      document.getElementById('httpPort').textContent = info.mixed_port;
      document.getElementById('socksPort').textContent = info.socks_port;
      document.getElementById('mixedPort').textContent = info.mixed_port;

      updateLanStatus(info.allow_lan);
      updateModeButtons(info.mode);
      updateButtonState();

      addLog(`✓ 已连接 Clash ${info.version}，API 端口 ${info.api_port}`, 'success');

      await refreshShareInfo();
    } else {
      setStatus('error', '未运行');
      addLog('✗ Clash 未运行，请先启动 Clash', 'error');
    }
  } catch (err) {
    setStatus('error', '未检测到');
    document.getElementById('clashVersion').textContent = '未连接';
    addLog(`✗ ${err}`, 'error');
    addLog('提示：请确保 Clash 已启动并开启了 RESTful API', 'warning');
  }
}

// ============ 切换局域网共享 ============

async function toggleLan() {
  const newState = !currentStatus.allowLan;
  addLog(`${newState ? '开启' : '关闭'}局域网共享...`, 'info');

  try {
    const result = await tauriInvoke('toggle_lan', { enable: newState });
    if (result) {
      currentStatus.allowLan = newState;
      updateLanStatus(newState);
      addLog(`✓ 已${newState ? '开启' : '关闭'}局域网共享`, 'success');
      showToast(`局域网共享已${newState ? '开启' : '关闭'}`);

      if (newState) {
        await refreshShareInfo();
        addLog('其他设备请设置 HTTP 代理为: ' + document.getElementById('shareHttp').textContent, 'info');
      }
    } else {
      addLog('✗ 操作失败，请检查 Clash 配置', 'error');
    }
  } catch (err) {
    addLog(`✗ ${err}`, 'error');
  }
}

// ============ 切换模式 ============

async function setMode(mode) {
  addLog(`切换代理模式: ${getModeLabel(mode)}...`, 'info');

  try {
    const result = await tauriInvoke('set_mode', { mode });
    if (result) {
      currentStatus.mode = mode;
      document.getElementById('clashMode').textContent = getModeLabel(mode);
      updateModeButtons(mode);
      addLog(`✓ 已切换为 ${getModeLabel(mode)} 模式`, 'success');
      showToast(`已切换为 ${getModeLabel(mode)} 模式`);
    }
  } catch (err) {
    addLog(`✗ ${err}`, 'error');
  }
}

// ============ 扫描设备 ============

async function scanDevices() {
  const btn = document.getElementById('btnScan');
  btn.disabled = true;
  btn.textContent = '扫描中...';
  addLog('正在扫描局域网设备...', 'info');

  try {
    const devices = await tauriInvoke('scan_devices');
    renderDevices(devices);
    addLog(`发现 ${devices.length} 个设备`, devices.length > 0 ? 'success' : 'warning');
  } catch (err) {
    addLog(`✗ ${err}`, 'error');
  } finally {
    btn.disabled = false;
    btn.textContent = '🔍 扫描设备';
  }
}

// ============ 刷新分享信息 ============

async function refreshShareInfo() {
  try {
    const info = await tauriInvoke('get_share_info');
    document.getElementById('lanIp').textContent = info.lan_ip;
    document.getElementById('shareHttp').textContent = info.http_url;
    document.getElementById('shareSocks').textContent = info.socks_url;
  } catch (err) {
    console.error('获取分享信息失败:', err);
  }
}

// ============ UI 更新 ============

function setStatus(type, text) {
  const dot = document.getElementById('statusDot');
  const textEl = document.getElementById('statusText');
  dot.className = 'status-dot ' + type;
  textEl.textContent = text;
}

function updateLanStatus(enabled) {
  const el = document.getElementById('clashLan');
  const btn = document.getElementById('btnToggleLan');

  if (enabled) {
    el.textContent = '✓ 已开启';
    el.style.color = 'var(--success)';
    btn.textContent = '关闭局域网共享';
    btn.classList.add('active');
  } else {
    el.textContent = '✗ 未开启';
    el.style.color = 'var(--text-secondary)';
    btn.textContent = '开启局域网共享';
    btn.classList.remove('active');
  }
}

function updateButtonState() {
  const btn = document.getElementById('btnToggleLan');
  btn.disabled = !currentStatus.running;
}

function updateModeButtons(activeMode) {
  document.querySelectorAll('.btn-mode').forEach(btn => {
    btn.classList.remove('active');
    if (btn.textContent === getModeLabel(activeMode)) {
      btn.classList.add('active');
    }
  });
}

function getModeLabel(mode) {
  const labels = { rule: '规则', global: '全局', direct: '直连' };
  return labels[mode] || mode;
}

function renderDevices(devices) {
  const container = document.getElementById('deviceList');

  if (devices.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <span class="empty-icon">📡</span>
        <p>未发现设备</p>
        <p class="empty-hint">请确保设备在同一局域网内</p>
      </div>
    `;
    return;
  }

  container.innerHTML = devices.map(dev => `
    <div class="device-item">
      <span class="device-icon">📱</span>
      <div class="device-info">
        <div class="device-name">${dev.ip}</div>
        <div class="device-detail">${dev.mac}</div>
      </div>
    </div>
  `).join('');
}

// ============ 复制功能 ============

function copyText(elementId) {
  const text = document.getElementById(elementId).textContent;
  if (!text || text === '—' || text === '未连接' || text === '未检测' || text === '检测中...') {
    return;
  }

  navigator.clipboard.writeText(text).then(() => {
    showToast('已复制到剪贴板');
    const btn = event.target;
    btn.classList.add('copied');
    btn.textContent = '✓';
    setTimeout(() => {
      btn.classList.remove('copied');
      btn.textContent = '复制';
    }, 1000);
  });
}

// ============ 日志 ============

function addLog(message, level = 'info') {
  const viewer = document.getElementById('logViewer');
  const now = new Date();
  const time = now.toTimeString().slice(0, 8);

  const icons = {
    info: 'ℹ',
    success: '✓',
    warning: '⚠',
    error: '✗',
  };

  const line = document.createElement('div');
  line.className = 'log-line';
  line.innerHTML = `
    <span class="log-time">[${time}]</span>
    <span class="log-icon ${level}">${icons[level] || '·'}</span>
    <span class="log-msg">${escapeHtml(message)}</span>
  `;

  viewer.appendChild(line);
  viewer.scrollTop = viewer.scrollHeight;
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// ============ Toast ============

let toastTimer = null;

function showToast(message) {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.classList.add('show');

  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.classList.remove('show');
  }, 2000);
}
