# CLAUDE.md — AI 开发上下文

## 项目概述

Mimonitor Toolbox 是一个通过无线 ADB 控制红米 G Pro 27U 显示器的桌面应用。

## 技术栈

- **后端**: Rust + Tauri v2 + tokio
- **前端**: React + TypeScript + shadcn/ui + Tailwind CSS
- **ADB**: adb_client crate (纯 Rust，无需 adb 二进制)
- **硬件控制**: MonitorTool.jar (MTK JNI + MiTV PM2 HIDL)

## 关键架构

### ADB 通信

显示器内置 Android 系统，通过无线 ADB (port 5555) 连接。两种控制方式:

1. **Android settings** (`settings get/put global`) — 标准设置
2. **JNI 调用** — 通过 `service call TvService` 执行 JAR 包中的 Java 类

### MonitorTool.jar

统一的硬件控制工具，处理 MTK JNI 寄存器读写和 LED HIDL 控制:
- 本地路径: `mimonitor/resources/MonitorTool.jar`
- 设备路径: `/sdcard/` → `/data/data/mitv.service/cache/`
- 自动部署: `ensure_jar()` 函数检查并推送
- 源码: `tools/monitor-tool/MonitorTool.java`

### LED 控制注意点

LED 通过 HIDL 直接控制硬件，**不会同步更新 Android settings 数据库**。每次 LED 操作后必须同时调用 `put_setting` 同步状态。

### 灯光模式

| 模式 | 设置值 | HIDL mode | 可调参数 |
|------|--------|-----------|----------|
| 关闭 | 4 | 0 | 无 |
| 照明 | 0 | 11 | 色温 (2700K/4000K/6500K) |
| 纯色 | 2 | 1 | 颜色 (冰蓝/流金/天青/草地/日落) |
| 屏幕同色 | 1 | 12 | 无 |
| 七彩循环 | 3 | 3 | 无 |

照明和纯色是不同模式，不可混用色温和颜色。

### 扫描器

`get_local_subnets()` 解析 `ifconfig` 输出，跳过虚拟接口 (utun/bridge/docker 等)，优先返回 `192.168.x.x` 子网。

### 前端状态管理

每个设置使用独立 `useState`，不用共享对象。切换模式时发送完整命令 (mode + params)，不依赖 `refresh()`。

## 常用命令

```bash
# 开发
cd mimonitor && cargo tauri dev

# 构建
cd mimonitor && cargo tauri build

# 测试
cd mimonitor/src-tauri && cargo test

# TypeScript 检查
cd mimonitor && npx tsc --noEmit
```

## 文件说明

| 文件 | 说明 |
|------|------|
| `mimonitor/src-tauri/src/adb/mod.rs` | AdbClient 核心 (shell/settings/batch) |
| `mimonitor/src-tauri/src/adb/jni.rs` | MonitorTool.jar JNI + LED 调用 |
| `mimonitor/src-tauri/src/adb/scanner.rs` | 内网扫描 |
| `mimonitor/src-tauri/src/commands/` | Tauri IPC 命令 |
| `mimonitor/src-tauri/src/socket/mod.rs` | TCP Socket JSON-RPC 服务 |
| `mimonitor/src-tauri/src/state/` | 应用状态 + 配置 |
| `mimonitor/src/pages/` | React 前端页面 |
| `tools/monitor-tool/MonitorTool.java` | 硬件控制工具源码 |
