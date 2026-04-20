# 局域网文件传输工具 PRD

## 一、项目概述

### 1.1 项目背景
在办公场景中，同事间经常需要共享文件。传统方式如微信传输、邮件附件存在文件大小限制、需要联网等问题。本项目旨在提供一个局域网内的快速、安全的文件传输解决方案。

### 1.2 目标用户
- 办公场景下的同事间文件共享
- 包括技术人员和非技术人员
- 需要简单易用的操作界面

### 1.3 核心价值
- 无需互联网连接，局域网内直接传输
- 支持大文件传输，无大小限制
- 自动发现设备，无需手动配置
- 接收确认机制，防止误发送

---

## 二、技术架构

### 2.1 技术选型

| 层级 | 技术 | 版本 | 说明 |
|------|------|------|------|
| 前端框架 | React | 18.x | 类型安全，组件化开发 |
| 构建工具 | Vite | 5.x | HMR快速，Tauri官方推荐 |
| UI组件库 | Ant Design | 5.x | 企业级设计，中文文档完善 |
| 状态管理 | Zustand | 4.x | 轻量简单 |
| 桌面框架 | Tauri | 2.x | 内置Rust后端，跨平台，体积小 |
| HTTP服务 | axum | 0.7 | Tokio生态，异步性能好 |
| 本地存储 | SQLite | - | 轻量级，无需额外服务 |
| 异步运行时 | Tokio | 1.x | Rust异步标准 |

### 2.2 为什么选择Tauri而非纯Web

| 功能需求 | 纯Web | Tauri |
|----------|-------|-------|
| UDP广播设备发现 | ❌ 浏览器禁止 | ✅ Rust实现 |
| HTTP服务器监听 | ❌ 浏览器无法 | ✅ axum实现 |
| 本地文件系统访问 | ❌ 受限 | ✅ 完全访问 |
| SQLite数据库 | ❌ 需要后端 | ✅ 本地存储 |
| 跨平台桌面应用 | ❌ | ✅ Windows/Mac/Linux |

---

## 三、核心功能模块

### 3.1 设备自动发现

**技术方案：UDP广播**

- 广播端口：3737
- 广播间隔：5秒
- 设备超时：15秒无响应标记离线

**消息格式：**
```json
{
  "msg_type": "announce",     // announce | response | bye
  "device_id": "uuid-string",
  "device_name": "设备名称",
  "ip": "192.168.1.x",
  "port": 8080,
  "timestamp": 1713600000
}
```

**流程说明：**
1. 应用启动后，broadcaster每5秒向255.255.255.255:3737发送announce消息
2. listener监听3737端口，收到announce后：
   - 发送`device-discovered`事件到前端显示设备
   - 回复response消息告知对方自己的存在
3. 前端通过Zustand状态管理维护设备列表
4. 超过15秒未收到某设备消息，标记为离线

### 3.2 HTTP文件传输服务

**服务端口：8080**

**API接口设计：**

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/transfer/request` | POST | 发送传输请求 |
| `/api/transfer/accept` | POST | 接收方接受 |
| `/api/transfer/reject` | POST | 接收方拒绝 |
| `/api/upload` | POST | 上传文件分片 |
| `/api/download/{file_id}` | GET | 下载文件（支持Range） |
| `/api/status/{transfer_id}` | GET | 查询传输状态 |
| `/api/transfer/cancel` | POST | 取消传输 |

### 3.3 接收确认流程

```
发送方                                    接收方
  │                                        │
  │  1. POST /transfer/request             │
  │  {file_info, device_id}                │
  │ ─────────────────────────────────────> │
  │                                        │
  │                  2. 弹窗确认/拒绝        │
  │                     [用户操作]          │
  │                                        │
  │  3. 接收方响应                          │
  │ <───────────────────────────────────── │
  │  {accept: true/false, transfer_id}     │
  │                                        │
  │  4. 若接受，开始传输                     │
  │ ─────────────────────────────────────> │
  │                                        │
```

### 3.4 断点续传

**实现方案：HTTP Range请求**

- 请求头：`Range: bytes=0-1023`
- 响应头：`Content-Range: bytes 0-1023/5000`
- 响应状态：`206 Partial Content`
- 临时文件：`.partial`后缀存储未完成文件

### 3.5 文件夹传输

**方案：保持目录结构 + 流式传输**

- 遍历文件夹生成文件树结构
- 传输时保持相对路径信息
- 接收方按相对路径写入，自动创建子目录

### 3.6 传输历史记录

**SQLite数据表设计：**

```sql
-- 设备表
CREATE TABLE devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ip TEXT NOT NULL,
    last_seen INTEGER NOT NULL
);

-- 传输记录表
CREATE TABLE transfers (
    id TEXT PRIMARY KEY,
    direction TEXT NOT NULL,  -- 'send' | 'receive'
    status TEXT NOT NULL,     -- 'pending'|'in_progress'|'completed'|'cancelled'|'rejected'
    peer_device_id TEXT NOT NULL,
    peer_device_name TEXT NOT NULL,
    total_size INTEGER NOT NULL,
    transferred_size INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    completed_at INTEGER
);

-- 文件记录表
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    transfer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT,
    size INTEGER NOT NULL,
    status TEXT NOT NULL
);
```

---

## 四、项目目录结构

```
lan-transmission/
├── src/                          # 前端源码
│   ├── main.tsx                  # 应用入口
│   ├── App.tsx                   # 根组件（路由+布局）
│   ├── components/               # 通用组件
│   │   ├── DeviceList/           # 设备列表
│   │   ├── FileDropZone/         # 文件拖拽上传区
│   │   ├── TransferProgress/     # 传输进度
│   │   ├── HistoryList/          # 历史记录列表
│   │   └── ConfirmDialog/        # 接收确认对话框
│   ├── pages/                    # 页面
│   │   ├── Home/                 # 主页（发送）
│   │   └── History/              # 历史记录页
│   ├── stores/                   # Zustand状态管理
│   │   ├── deviceStore.ts        # 设备状态
│   │   ├── transferStore.ts      # 传输状态
│   │   └── historyStore.ts       # 历史记录状态
│   ├── hooks/                    # 自定义Hooks
│   │   └── useDeviceDiscovery.ts # 设备发现监听
│   ├── types/                    # TypeScript类型定义
│   └── styles/                   # 样式文件
├── src-tauri/                    # Tauri后端
│   ├── Cargo.toml                # Rust依赖配置
│   ├── tauri.conf.json           # Tauri应用配置
│   ├── capabilities/             # Tauri权限配置
│   │   └ default.json            # 默认权限
│   └── src/
│       ├── main.rs               # 应用入口
│       ├── lib.rs                # 库导出
│       ├── discovery/            # 设备发现模块
│       │   ├── mod.rs
│       │   ├── broadcaster.rs    # UDP广播发送
│       │   ├── listener.rs       # UDP监听
│       │   └── device_registry.rs # 设备注册表
│       ├── transfer/             # 文件传输模块
│       │   ├── mod.rs
│       │   ├── http_server.rs    # HTTP服务器
│       │   ├── upload_handler.rs # 上传处理
│       │   └ download_handler.rs # 下载处理
│       ├── history/              # 历史记录模块
│       │   ├── mod.rs
│       │   ├── database.rs       # SQLite操作
│       │   └ models.rs           # 数据模型
│       ├── confirm/              # 接收确认模块
│       │   ├── mod.rs
│       │   └ protocol.rs         # 确认协议
│       └── icons/                # 应用图标
├── package.json
├── vite.config.ts
├── tsconfig.json
└ └── README.md
```

---

## 五、开发进度

### 5.1 已完成功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 前端框架搭建 | ✅ 完成 | React + Ant Design + Zustand |
| Tauri桌面应用 | ✅ 完成 | Windows平台运行正常 |
| UDP设备发现 | ✅ 完成 | 广播发送+监听+前端监听 |
| HTTP服务器 | ✅ 框架完成 | axum服务器已启动，API路由已定义 |
| SQLite数据库 | ✅ 完成 | 表结构已创建 |
| 接收确认协议 | ✅ 框架完成 | 消息格式已定义 |

### 5.2 待完成功能

| 功能 | 优先级 | 说明 |
|------|--------|------|
| 文件上传实现 | P0 | upload_handler完整实现 |
| 文件下载实现 | P0 | download_handler完整实现 |
| 断点续传 | P1 | Range请求处理 |
| 文件夹传输 | P1 | 目录遍历+结构保持 |
| 传输进度实时更新 | P1 | 前端进度显示 |
| 接收确认前端UI | P0 | ConfirmDialog完整实现 |
| 历史记录页面 | P1 | History页面数据展示 |
| 传输取消 | P2 | 中断传输逻辑 |

---

## 六、界面设计

### 6.1 主页布局

```
┌─────────────────────────────────────────────────────────┐
│  [侧边栏]                                                │
│  ┌──────────┐                                           │
│  │ 文件传输 │    ┌─────────────┐  ┌─────────────────┐   │
│  ├──────────┤    │ 在线设备(2) │  │ 发送文件        │   │
│  │ 发送文件 │    │             │  │                 │   │
│  │ 传输历史 │    │ ○ 张三电脑  │  │ [拖拽区域]      │   │
│  └──────────┘    │ ○ 李四电脑  │  │                 │   │
│                  │             │  ├─────────────────┤   │
│                  └─────────────┘  │ 传输进度        │   │
│                                    │                 │   │
│                                    │ 发送: 50%       │   │
│                                    │ 接收: 完成      │   │
│                                    └─────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 6.2 接收确认弹窗

```
┌─────────────────────────────────┐
│       接收文件请求               │
├─────────────────────────────────┤
│                                 │
│  张三想要发送以下文件：          │
│                                 │
│  ○ project.zip (50MB)           │
│  ○ readme.md (2KB)              │
│                                 │
│  总大小: 50.02 MB               │
│                                 │
├─────────────────────────────────┤
│         [拒绝]  [接收]          │
└─────────────────────────────────┘
```

---

## 七、安全考虑

### 7.1 当前安全措施
- 接收确认机制：防止误发送
- 局域网限制：仅同一网络可发现

### 7.2 后续安全增强（可选）
- 传输密码保护
- 设备绑定白名单
- 文件类型过滤

---

## 八、测试计划

### 8.1 功能测试

| 测试项 | 测试方法 |
|--------|----------|
| 设备发现 | 启动两个实例，验证互相发现 |
| 单文件传输 | 发送10MB文件，验证完整性 |
| 大文件传输 | 发送1GB文件，验证稳定性 |
| 断点续传 | 中断后重新下载，验证恢复 |
| 文件夹传输 | 发送含子目录文件夹 |
| 接收确认 | 验证接受/拒绝流程 |
| 历史记录 | 检查传输后记录完整性 |

---

## 九、后续迭代方向

1. **跨平台支持** - macOS、Linux版本
2. **移动端支持** - iOS/Android companion app
3. **传输加密** - 可选的文件加密传输
4. **多选发送** - 同时向多个设备发送
5. **剪贴板共享** - 文本内容快速共享