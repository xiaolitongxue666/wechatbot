# AGENTS.md — WeChatBot Project Conventions

## Project Overview

Monorepo centered on the **Rust** iLink Bot stack (SDK + admin + admin web UI) at the **repository root**. Reference SDKs and archived projects live in sibling directories only.

```
wechatbot/                    # Rust 主工程（仓库根 — 在此执行 cargo）
├── src/, config/, migrations/, tests/, examples/
├── admin/web/                # Vue 管理前端
├── deploy/                   # docker-compose
├── tools/scripts/, tools/skill/
├── docs/                     # 文档（protocol、architecture、rust/）
├── reference-sdks/           # 参考 SDK
└── legacy/                   # 归档子项目
```

**Agent 必读：** [rust/agent-memory.md](rust/agent-memory.md)、[rust/troubleshooting.md](rust/troubleshooting.md)

## Working Directory

| Component | Working Directory |
|-----------|-------------------|
| **Rust 主工程** | **仓库根**（`cargo build`、`bash tools/scripts/start.sh`） |
| Admin 前端 | `admin/web/` |
| Node.js 参考 SDK | `reference-sdks/nodejs/` |
| Python 参考 SDK | `reference-sdks/python/` |
| Go 参考 SDK | `reference-sdks/golang/` |
| Pi Agent | `legacy/pi-agent/` |

**禁止**在已删除的 `rust/` 过渡目录或旧路径（`scripts/`、`web-admin/`、`doc/`）下执行命令。

## Commands

### Rust（仓库根）

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Test (unit / CI 默认) | `cargo test --lib` 或 `bash tools/scripts/test.sh` |
| Test (admin HTTP + PG) | `WECHATBOT_TEST_DATABASE_URL=... cargo test --test admin_frontend` |
| Test (full + Docker) | `bash tools/scripts/test_all.sh`（耗时可数分钟；跑前需 `admin/web/dist`） |
| Test (backend gate) | `bash tools/scripts/test/run_backend_tests.sh` |
| Test (E2E, 勿卡死) | `bash tools/scripts/test/run_e2e.sh` |
| Run admin | `cargo run --bin admin` 或 `bash tools/scripts/admin.sh start` |
| Run worker | `cargo run --bin worker` |
| Start env (dev, mock) | `bash tools/scripts/start.sh` |
| Start env (deploy, no mock) | `bash tools/scripts/start.sh --deploy` |
| Dev verify | `bash tools/scripts/dev.sh` |
| Web admin UI | `cd admin/web && npm run build` |

- Binaries: `admin`, `worker`; examples: `echo_bot`, `multi_bot_runtime`
- Config: `config/app.toml` + `.env`
- Admin URL: `http://127.0.0.1:8787/admin`
- CI: `cargo build && cargo test --lib` at repo root

### Admin 启动前检查

1. Docker PG/Redis 已启动（`bash tools/scripts/services.sh up`）
2. `admin/web/dist` 已构建
3. `.env` 存在（可从 `.env.example` 复制）
4. 8787 端口空闲（占用会导致 bind 失败，见 troubleshooting）

### Node.js (`reference-sdks/nodejs/`)

| Task | Command |
|------|---------|
| Install | `npm install` |
| Build | `npm run build` |
| Test | `npx vitest run` (NODE_OPTIONS=--experimental-vm-modules) |

### Python (`reference-sdks/python/`)

| Task | Command |
|------|---------|
| Install | `pip install -e ".[dev]"` |
| Test | `pytest` |

### Go (`reference-sdks/golang/`)

| Task | Command |
|------|---------|
| Build | `go build ./...` |
| Test | `go test ./...` |

### Pi Agent (`legacy/pi-agent/`)

| Task | Command |
|------|---------|
| Install | `npm install` |
| Lint | `tsc --noEmit` |

- Depends on npm package `@wechatbot/wechatbot` (not monorepo path)

## Scope Rules for Agents

- **默认只改仓库根** Rust / `admin/web` / `tools/scripts` / `docs/rust`
- **不要改** `reference-sdks/`、`legacy/` 除非任务明确涉及参考 SDK 或归档项目
- 结构性变更：同步 `docs/rust/README.md`、`docs/architecture.md`、根 `README.md` stub

## Code Conventions (All SDKs)

- **Section separators**: ASCII-art comment blocks between logical sections
- **Error hierarchy**: `WeChatBotError` with `code`; `ApiError` session expiry `errcode === -14`
- **Logging**: `[wechatbot]` prefix to stderr
- **Credentials**: `~/.wechatbot/credentials.json`

### Rust Specific

- `thiserror`, `async_trait`, `serde` camelCase, `Arc<RwLock<>>`, `tracing`
- Structural doc updates: `docs/rust/architecture.md`, `docs/rust/configuration.md`, `docs/rust/testing.md`, etc.

## TDD / Testing

| 变更范围 | 最低验证 |
|----------|----------|
| Rust `src/` | `cargo test --lib` |
| `admin/web/` | 先 `npm run build`，再 `bash tools/scripts/test/run_e2e.sh` |
| `reference-sdks/nodejs` | `npx vitest run` |

## Commit / PR Guidelines

- Concise messages describing "why"
- Never commit secrets, `.env`, logs, pids, `target/`, `node_modules/`, `admin/web/dist`
- Each published package versions independently
