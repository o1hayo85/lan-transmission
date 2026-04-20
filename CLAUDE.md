# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Tauri 2.x 的局域网文件传输桌面应用。前端使用 React + TypeScript，后端使用 Rust。

## 常用命令

```bash
# 安装依赖
npm install

# 启动前端开发服务器（端口 3000）
npm run dev

# 构建前端
npm run build

# 启动 Tauri 开发模式（同时启动前端和后端）
npm run tauri dev

# 构建 Tauri 应用（生成安装包）
npm run tauri build
```

## 架构结构

### 前端 (src/)

- **状态管理**: Zustand stores (`stores/*.ts`) - 管理设备和传输状态
- **页面**: `pages/Home`（发送文件）、`pages/History`（传输历史）
- **组件**: `components/DeviceList`、`FileDropZone`、`TransferProgress`、`ConfirmDialog`
- **路由**: React Router，侧边栏导航
- **UI库**: Ant Design

### 后端 (src-tauri/src/)

核心模块：

- **discovery** (`discovery/mod.rs`): UDP广播设备发现
  - `broadcaster.rs`: 每5秒广播本机信息到 UDP 3737 端口
  - `listener.rs`: 监听 UDP 3737 接收其他设备广播
  - `device_registry.rs`: 设备注册表管理

- **transfer** (`transfer/mod.rs`): HTTP 文件传输服务器（端口 8080）
  - `http_server.rs`: Axum HTTP 服务器，提供 API 端点
  - `upload_handler.rs`: 处理上传请求
  - `download_handler.rs`: 处理下载请求

- **history** (`history/mod.rs`): SQLite 数据库存储传输历史
  - `database.rs`: 数据库初始化和操作
  - `models.rs`: 数据模型定义

- **confirm** (`confirm/mod.rs`): 传输确认协议模块

### Tauri 事件通信

后端通过 `app_handle.emit()` 发送事件到前端：
- `device-discovered`: 发现新设备
- `device-lost`: 设备离线
- `transfer-request`: 收到传输请求（前端显示确认对话框）
- `transfer-started`: 传输开始
- `transfer-progress`: 传输进度更新
- `transfer-completed`: 传输完成
- `transfer-rejected`: 传输被拒绝

前端通过 `@tauri-apps/api` 的 `listen` 监听这些事件。

## 关键端口

- UDP 3737: 设备发现广播
- HTTP 8080: 文件传输 API

## API 端点

```
POST /api/transfer/request  - 发送传输请求
POST /api/transfer/accept   - 接受传输
POST /api/transfer/reject   - 拒绝传输
POST /api/upload            - 上传文件
GET  /api/download/:file_id - 下载文件
GET  /api/status/:transfer_id - 查询传输状态
POST /api/transfer/cancel   - 取消传输
```

## 开发注意事项

- 前端路径别名 `@` 映射到 `src/`
- Rust 代码修改后 Tauri 会自动重新编译
- 设备发现基于 UDP 广播，在同一局域网内工作
- 历史记录使用 SQLite（bundled 模式，无需额外安装）