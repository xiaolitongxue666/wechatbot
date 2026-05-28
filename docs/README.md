# WeChatBot

<p align="center">
  <strong>微信 iLink Bot — Rust 主工程 + 多语言参考 SDK</strong>
</p>

[English](README.en.md)

仓库根目录为 **Rust 协议栈与 Admin 服务**；管理前端在 [`admin/web/`](../admin/web/)。Node / Python / Go 在 [`reference-sdks/`](../reference-sdks/)，Pi 扩展等在 [`legacy/`](../legacy/)。

## 目录结构

```
wechatbot/                 # Rust 主工程（在此 cargo build）
├── src/, config/, migrations/, tests/, examples/
├── admin/web/             # Vue 管理前端
├── deploy/                # docker-compose（PG/Redis/MinIO）
├── tools/scripts/         # 运维脚本
├── tools/skill/           # Agent 操作剧本
├── docs/                  # 文档（本目录）
│   ├── protocol.md        # iLink 协议
│   ├── architecture.md    # 全仓架构
│   └── rust/              # Rust 运维、部署、排障
├── reference-sdks/        # 参考 SDK
└── legacy/                # 归档子项目
```

## 快速开始

**前置：** Docker Desktop、Rust toolchain、[Bun](https://bun.sh)（脚本构建前端）

```bash
# 仓库根目录
cp .env.example .env    # 首次

# 测试/演示（带 mock bot 与会话数据）
bash tools/scripts/start.sh

# 部署（仅 migrate，不灌 mock）
bash tools/scripts/start.sh --deploy
```

管理界面：**http://127.0.0.1:8787/admin**

## 技术栈

| 层级 | 选型 | 说明 |
|------|------|------|
| 后端 | Rust + Tokio + Axum | 二进制 `admin` / `worker`，见 [`rust/architecture.md`](rust/architecture.md) |
| 数据访问 | **SQLx**（非 ORM） | 手写 SQL + Repository；迁移 `migrations/*.sql` |
| 数据库 | PostgreSQL 16 | Bot、会话、消息、RBAC |
| 缓存/队列 | Redis 7 | 事件队列、会话在线状态 |
| 媒体 | localfs / MinIO | 可选 S3 兼容存储 |
| 前端 | Vue 3 + TS + Vite + Bun | 目录 `admin/web/`，E2E 用 Playwright |

全仓多语言对照：[architecture.md](architecture.md)

```bash
bash tools/scripts/admin.sh start   # 仅启动 admin（依赖已运行时）
bash tools/scripts/status.sh
bash tools/scripts/admin.sh stop
```

仅开发前端：

```bash
cd admin/web && npm run dev   # http://127.0.0.1:5174/admin/
```

## 安装已发布包

| 组件 | 安装 |
|------|------|
| Rust crate | `cargo add wechatbot` |
| Node 参考 SDK | `npm install @wechatbot/wechatbot` |
| Python 参考 SDK | `pip install wechatbot-sdk` |
| Go 参考 SDK | `go get github.com/corespeed-io/wechatbot/golang` |
| Pi 扩展 | `pi install npm:@wechatbot/pi-agent` |

预编译 echo-bot：根目录 `install.sh` / `install.ps1`（见 [Releases](https://github.com/corespeed-io/wechatbot/releases)）

## CI 与发布

| 组件 | 路径 | 门禁命令 |
|------|------|----------|
| Rust | 仓库根 | `cargo build && cargo test --lib` |
| Node.js | `reference-sdks/nodejs/` | `npx vitest run` |
| Python | `reference-sdks/python/` | `pytest` |
| Go | `reference-sdks/golang/` | `go test ./...` |
| Admin 前端 | `admin/web/` | `bash tools/scripts/test/run_e2e.sh` |

发布 tag：`node-v*` / `py-v*` / `pi-agent-v*` / 二进制 `v*`。Workflow 见 `.github/workflows/`。

## 文档索引

| 文档 | 说明 |
|------|------|
| [rust/README.md](rust/README.md) | Rust 运维、部署、测试 |
| [rust/troubleshooting.md](rust/troubleshooting.md) | 排障（含 2026-05 重组） |
| [rust/agent-memory.md](rust/agent-memory.md) | Agent 记忆、**问题与解法总表** |
| [protocol.md](protocol.md) | iLink 协议 |
| [architecture.md](architecture.md) | 全仓架构 |
| [rust/architecture.md](rust/architecture.md) | **Rust 前后端技术栈与模块** |
| [AGENTS.md](AGENTS.md) | 开发约定（含 Agent） |
| [../reference-sdks/README.md](../reference-sdks/README.md) | 参考 SDK 索引 |
| [../legacy/README.md](../legacy/README.md) | 归档子项目 |

## 测试

```bash
cargo test --lib
bash tools/scripts/test.sh
bash tools/scripts/test/run_e2e.sh
```

集成测试需 PostgreSQL：见 [rust/testing.md](rust/testing.md)。

## Rust 主工程摘要

- **Binaries：** `admin`、`worker`（`cargo run --bin admin`）
- **Examples：** `echo_bot`、`multi_bot_runtime`
- **配置：** `config/app.toml`，环境变量见 `.env.example`

详细 API 与模块说明见 [rust/code-analysis.md](rust/code-analysis.md) 及 [rust/architecture.md](rust/architecture.md)。

## License

MIT
