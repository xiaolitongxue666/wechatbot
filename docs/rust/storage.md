# 聊天记录与媒体存储

## 聊天记录
- 表：`chat_messages`
- 保存字段：`event_id`、`session_id`、`tenant_id`、`from_user_id`、`content_type`、`text_content`、`raw_payload_json`、`received_at`

## 媒体策略
- 媒体二进制不直接入数据库。
- 先计算 `sha256`，再写介质存储（本地目录或 S3 兼容存储）。
- 元数据写入 `chat_media`，记录 `storage_backend`、`storage_key`、`size_bytes`、`sha256`。

## 会话状态
- Redis 保存在线状态和心跳时间戳：
  - `wechatbot:session:{session_id}:online`
  - `wechatbot:session:{session_id}:heartbeat`
