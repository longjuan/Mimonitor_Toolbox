# Mimonitor Toolbox

红米 G Pro 27U 显示器控制工具 — 通过无线 ADB 连接并调优您的 MiniLED 旗舰显示器。

> 基于 [YiHooong/Mimonitor_Toolbox](https://github.com/YiHooong/Mimonitor_Toolbox) 重写的 Tauri 桌面版本。

## 功能

| 类别 | 功能 |
|------|------|
| **画面设置** | 模式(标准/游戏/电影)、背光、黑色级别、对比度、饱和度、色调、锐度、色温(冷/标准/暖/原色/自定义)、RGB 增益、精密控光(关/低/中/高)、动态清晰度、响应时间、色域(sRGB/DCI-P3/AdobeRGB/BT2020/BT709) |
| **游戏模式** | 准星(5种)、动态准星、狙击镜(1.1x-2.0x)、夜视、320Hz 竞技、FreeSync Premium Pro、FPS 计数器、秒表、定时器 |
| **输入源** | HDMI 1/2、DP、USBC 一键切换 |
| **屏幕灯光** | 照明(2700K/4000K/6500K)、纯色(冰蓝/流金/天青/草地/日落)、屏幕同色、七彩循环 |
| **虚拟遥控器** | 电源、Home、菜单、返回、D-Pad、音量 |
| **AI 集成** | MCP Server + CLI + TCP Socket，支持 Claude Code 等 AI 工具直接控制 |
| **跨平台** | macOS + Windows (Linux 可选) |

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Rust |
| GUI | Tauri v2 + React + TypeScript + shadcn/ui |
| ADB | adb_client (纯 Rust ADB 协议，无需 bundled adb 二进制) |
| 硬件控制 | MonitorTool.jar (MTK JNI + MiTV PM2 HIDL) |
| IPC | TCP Socket (JSON-RPC) + MCP (stdio) |
| 异步 | tokio |
| 打包 | macOS DMG / Windows NSIS / Linux AppImage |

## 快速开始

### 环境要求

- Rust 1.77+
- Node.js 18+

### 开发模式

```bash
cd mimonitor
npm install
cargo tauri dev
```

### 构建应用

```bash
cd mimonitor
npm install
cargo tauri build
```

产物位于 `mimonitor/src-tauri/target/release/bundle/`。

### 运行测试

```bash
cd mimonitor/src-tauri
cargo test
```

## 使用方式

### GUI 应用

启动后自动扫描内网，发现一台设备时自动连接。支持：
- 画面参数实时调节 (300ms 防抖)
- 灯光模式切换 (即时响应)
- 虚拟遥控器
- 日志记录与导出

### CLI 命令

应用启动后会监听本地 TCP 端口 (端口号写入 `~/.mimonitor/port`)，CLI 通过 JSON-RPC 协议通信。

支持的 RPC 方法:

| 方法 | 参数 | 说明 |
|------|------|------|
| `scan` | - | 扫描内网设备 |
| `connect` | `{ip}` | 连接设备 |
| `disconnect` | - | 断开连接 |
| `get_status` | - | 获取连接状态 |
| `get_setting` | `{key}` | 读取设置 |
| `set_setting` | `{key, value}` | 写入设置 |
| `get_picture_settings` | - | 批量读取画面设置 |
| `get_game_settings` | - | 批量读取游戏设置 |
| `get_light_settings` | - | 批量读取灯光设置 |
| `set_led` | `{mode, brightness?, color?, color_temp?}` | 控制 LED |
| `set_input_source` | `{source}` | 切换输入源 |
| `send_key` | `{key}` | 发送遥控器按键 |

### Claude Code 集成 (MCP)

在项目根目录创建 `.claude/settings.json`:

```json
{
  "mcpServers": {
    "mimonitor": {
      "command": "mimonitor",
      "args": ["--mcp"]
    }
  }
}
```

MCP 工具:

| 工具 | 说明 |
|------|------|
| `scan` | 扫描内网 |
| `connect` | 连接设备 |
| `set_picture_mode` | 切换画面模式 |
| `set_picture_setting` | 设置画面参数 |
| `set_local_dimming` | 设置精密控光 |
| `set_led` | 控制 LED 灯光 |
| `set_input_source` | 切换输入源 |
| `send_remote_key` | 发送遥控器按键 |
| `get_monitor_status` | 获取显示器状态 |

使用示例:
```
用户: "帮我把显示器调成游戏模式，背光 80"
Claude Code → set_picture_mode(game) + set_picture_setting(backlight, 80)

用户: "任务完成了，亮灯"
Claude Code → set_led(solid, brightness=14, color=ice-blue)
```

## 实现原理

```
PC (Mimonitor)
  │
  ├─ ADB Wireless (port 5555) ──► 显示器 Android 系统
  │
  ├─ settings get/put ──► 读写 Android Global Settings
  │
  └─ MonitorTool.jar ──► MTK JNI + MiTV PM2 HIDL
      ├─ get/set/getMinMax/setColorGains/dump
      └─ led off/ambient/cycle/lighting/solid/raw
```

## 项目结构

```
Mimonitor_Toolbox/
├── mimonitor/                  # 应用主体
│   ├── src-tauri/              # Rust 后端
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── adb/            # ADB 客户端、JNI、LED、扫描
│   │       ├── commands/       # Tauri IPC 命令
│   │       ├── socket/         # TCP Socket 服务
│   │       ├── mcp/            # MCP 工具定义
│   │       ├── hdr/            # HDR 检测
│   │       └── state/          # 应用状态 + 配置
│   ├── src/                    # React 前端
│   │   ├── pages/              # 功能页面
│   │   ├── components/ui/      # UI 组件
│   │   └── styles/             # 样式
│   └── resources/              # MonitorTool.jar、APK
├── tools/
│   └── monitor-tool/           # MonitorTool.java 源码
├── .github/workflows/          # CI/CD
├── CLAUDE.md                   # AI 开发上下文
└── README.md
```

## License

MIT
