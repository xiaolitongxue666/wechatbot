# Rust 多 Bot 架构

## 模块边界
- `core`：`protocol/crypto/error/types`，专注协议与核心数据结构。
- `runtime`：会话管理、消息 ingest、队列与转发编排。
- `admin`：Axum 管理面路由、JSON API、权限控制与 Vue 静态资源托管。
- `infra`：配置加载、日志初始化、数据库/缓存/媒体适配。
- `storage`：封装 Postgres、Redis、媒体存储实现。
- `forwarder`：消费队列并转发外部服务，负责重试、策略检查与失败隔离。

## 数据流
1. Bot 收到消息。
2. `MessageIngestor` 标准化为 `EventEnvelope`。
3. 文本/结构化消息写 `chat_messages`。
4. 媒体下载后写介质存储，元数据写 `chat_media`。
5. 事件推送到队列。
6. `ForwarderWorker` 读取并签名转发下游服务。
7. 转发前按 `bot_forward_policies` 做策略检查（开关 + allowed_targets）。

## 管理端权限模型
- `admin_users`：管理员身份与 token 哈希
- `admin_permissions`：角色到权限点映射
- `admin_user_bot_scopes`：用户可访问 bot 范围
- `bot_forward_policies`：Bot 级转发策略（可启停、目标白名单）
