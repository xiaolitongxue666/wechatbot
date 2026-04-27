# Rust 多 Bot 架构

## 模块边界
- `protocol_sdk`：`bot/protocol/types`，专注微信协议通信。
- `session_manager`：维护每个 `session_id` 的生命周期和运行状态。
- `message_ingest`：标准化消息事件，落库并写入队列。
- `storage`：封装 Postgres、Redis、媒体存储实现。
- `forwarder`：消费队列并转发外部服务，负责重试与失败隔离。
- `runtime`：整合配置、注册 bot、启动会话和 worker。

## 数据流
1. Bot 收到消息。
2. `MessageIngestor` 标准化为 `EventEnvelope`。
3. 文本/结构化消息写 `chat_messages`。
4. 媒体下载后写介质存储，元数据写 `chat_media`。
5. 事件推送到队列。
6. `ForwarderWorker` 读取并签名转发下游服务。
