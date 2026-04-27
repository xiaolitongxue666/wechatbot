# AGENTS.md — Rust SDK 子项目规范（Cursor / OpenCode）

本规范作用域仅限 `rust/` 子目录。

## 1. 继承关系

- 继承仓库根规范：`/Users/liyong/Code/AI/wechatbot/AGENTS.md`
- 若根规范与本文件冲突，在 `rust/` 范围内以本文件为准。
- 本文件不覆盖 Node.js / Python / Go / pi-agent 子项目规则。

## 2. 工作目录策略

所有开发命令必须在 `rust/` 目录执行，不在仓库根目录执行 Rust 命令。

## 3. 命令矩阵

### 3.1 构建与运行

- `cargo build`
- `cargo run --bin admin`
- `cargo run --example echo_bot`
- `bash scripts/start.sh`

### 3.2 测试

- `cargo test`
- `bash scripts/test.sh`
- `bash scripts/test_all.sh`
- `bash scripts/dev.sh`

### 3.3 执行优先级

- 优先使用 `scripts/*.sh`（统一依赖与环境准备）。
- 需要单步调试时再直接使用 `cargo` 命令。

## 4. 最低交付门槛

在提交 Rust 改动前，最少满足：

1. `cargo build`
2. `cargo test`

若改动涉及存储、转发或完整流程，增加执行：

- `bash scripts/test_all.sh`

## 5. 配置与密钥边界

- 主配置文件：`config/app.toml`
- 默认凭据文件：`~/.wechatbot/credentials.json`
- 禁止提交真实密钥、token、账号凭据到仓库。
- 文档与示例仅使用占位符，不写生产凭据。

## 6. Rust 代码约定

- 错误统一归入 `WeChatBotError` 分层处理。
- 序列化字段保持 `camelCase` 约定（serde 配置一致）。
- 并发共享状态优先使用 `Arc<RwLock<...>>`，避免长时间持锁执行 I/O。
- 日志统一使用 `tracing`，关键事件需包含可定位上下文（如 bot/session 相关标识）。

## 7. 稳定性与性能守则

- 登录、轮询、转发链路变更需保留会话过期恢复能力。
- 重试逻辑必须有上限与退避，避免无限重试放大故障。
- 对队列/存储的高频路径改动要关注锁竞争与阻塞风险。

## 8. 文档联动

涉及 Rust 结构性改动时，需要同步更新文档：

- `doc/architecture.md`
- `doc/configuration.md`
- `doc/storage.md`
- `doc/testing.md`
- `doc/code-analysis.md`
- `doc/README.md`

## 9. OpenCode / Cursor 使用建议

- 在生成或执行命令前，先确认当前目录为 `rust/`。
- 优先引用本文件命令矩阵，减少跨语言命令误用。
- 修改配置、存储、会话、转发相关实现时，优先补测试与文档再交付。
