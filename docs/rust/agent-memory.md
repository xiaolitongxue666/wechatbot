# Agent 项目记忆

面向 Cursor / 其他编码 Agent 的**本仓库**上下文（非全局 skill）。完整规范见 [AGENTS.md](../AGENTS.md)、Cursor 规则 [`.cursor/rules/wechatbot.mdc`](../../.cursor/rules/wechatbot.mdc)。

## 2026-05 目录结构（v2，已落地）

| 路径 | 含义 |
|------|------|
| **仓库根** | Rust 主工程：`Cargo.toml`、`src/`、`config/`、`migrations/`、`tests/` |
| `admin/web/` | Vue 管理前端（原 `web-admin/`） |
| `deploy/` | `docker-compose.*.yml`（PG / Redis / MinIO） |
| `tools/scripts/` | 运维脚本（原根目录 `scripts/`） |
| `tools/skill/` | Agent 操作剧本 |
| `docs/` | **唯一文档根**：`protocol.md`、`architecture.md`、`rust/*` |
| `reference-sdks/` | Node / Python / Go 参考 SDK |
| `legacy/` | 归档：pi-agent、ai-app、devtools-bookmark、webchat |

**已删除/不再存在：** 根目录 `scripts/`、`web-admin/`、`doc/`、`wechat_official/`、`rust/` 过渡目录。

**易混淆：**

| 名称 | 实际含义 |
|------|----------|
| `admin/web/` | Vue SPA，由 Axum 挂载到 `/admin` |
| `wechat_official/` | **已删除**；微信官方 `user_id` 说明见 [wechat-official-notes.md](wechat-official-notes.md) |
| 根 `README.md` / `AGENTS.md` | **stub**，正文在 `docs/` |

## 禁止事项

- 不要在已删除的 `rust/` 或旧路径 `scripts/`、`web-admin/`、`doc/` 下执行命令
- 不要默认修改 `legacy/`、`reference-sdks/`（除非任务明确要求）
- 不要提交：`.env`、`*.log`、`*.pid`、`node_modules/`、`admin/web/dist/`、`target/`、`**/dist/`、`**/*.exe`
- **不要**在 Agent 会话中长时间前台跑 `test_all.sh` / 内嵌 Playwright `webServer`（易表现为卡死）——用下方推荐脚本

## 默认验证命令

```bash
# 仓库根 — 单元测试（CI 默认门禁）
cargo build && cargo test --lib

# 后端集成（Docker PG:5433，耗时可数分钟，非卡死）
cd admin/web && npm run build   # admin_frontend 依赖 dist，必须先构建
bash tools/scripts/test/run_backend_tests.sh

# 前端 E2E（preview 后台 + 自动清理，勿卡死）
bash tools/scripts/test/run_e2e.sh

# 参考 Node SDK
cd reference-sdks/nodejs && npx vitest run
```

## Admin 启动检查清单

1. Docker：PostgreSQL 5432、Redis 6379（`bash tools/scripts/services.sh up`）
2. `cp .env.example .env`，`WECHATBOT_WEB_ADMIN_DIST_DIR=./admin/web/dist`
3. `cd admin/web && npm run build`
4. 8787 端口未被占用（占用会导致 bind 失败但 healthz 可能仍来自旧进程）
5. `bash tools/scripts/admin.sh start` → http://127.0.0.1:8787/admin

详见 [troubleshooting.md](troubleshooting.md)。

## 发布路径

| 包 | 目录 | Tag |
|----|------|-----|
| `wechatbot` (crate) | 仓库根 | （手动 crates.io） |
| `@wechatbot/wechatbot` | `reference-sdks/nodejs/` | `node-v*` |
| `wechatbot-sdk` | `reference-sdks/python/` | `py-v*` |
| `@wechatbot/pi-agent` | `legacy/pi-agent/` | `pi-agent-v*` |
| echo-bot 二进制 | 仓库根 / golang | `v*` |

---

## 问题与解法总表（2026-05 重组 + 测试）

### 启动与数据

| 场景 | 命令 | 是否 mock |
|------|------|-----------|
| 本地测试 / Admin 演示 | `bash tools/scripts/start.sh` | **是**（`db.sh seed`） |
| 全栈（含 worker） | `bash tools/scripts/start_all.sh` | **是**（默认） |
| 部署 / 生产 | `bash tools/scripts/start.sh --deploy` 或 `start_all.sh --deploy` | **否**（仅 migrate） |
| 一键关闭 | `bash tools/scripts/stop.sh` 或 `stop_all.sh`（`-v` 删卷） | — |

**禁止**对生产库执行 `db.sh seed`。`admin.sh start` 本身不灌种，只启动进程。

`start_all.sh` / `start.sh` 启动后**立即退出**；admin、worker、Docker 在后台 detach。Ctrl+C 只能中断前台 bash，不能关停后台服务 → 用 `stop_all.sh` / `stop.sh`。

### 目录与迁移

| 现象 | 原因 | 解法 |
|------|------|------|
| 根目录 `cargo build` 失败、`src/` 为空 | Rust 主工程未完全上提到根，代码仍在旧 `rust/` | 确认 `src/`、`admin/web/`、`tools/scripts/` 在仓库根 |
| `bash scripts/*` 找不到 | 脚本已迁至 `tools/scripts/` | `bash tools/scripts/start.sh` 等 |
| `_common.sh` 路径指向 `tools/` | 重组后脚本深度变化，旧推导 `dirname×2` 不够 | `PROJECT_ROOT` = `tools/scripts` **上两级**（见 `_common.sh`） |
| `run_backend_tests.sh` 找不到 `_common.sh` | 脚本在 `tools/scripts/test/`，需 **上三级** 才到仓库根 | 已修复：`PROJECT_ROOT="$(cd .../../../.. && pwd)"` |
| `doc/` 与 `docs/` 双文档根 | 历史遗留 | 统一为 `docs/` + `docs/rust/` |
| SDK 路径混乱 | 根目录 `nodejs/` 与 `reference-sdks/` 并存 | 参考 SDK **仅**在 `reference-sdks/` |
| `wechat_official/` 与 `web-admin` 能否合并 | 前者是协议文档、后者是 SPA，无关 | 删 `wechat_official/`；前端用 `admin/web/` |

### Admin 与前端

| 现象 | 原因 | 解法 |
|------|------|------|
| `/admin` 404，`/healthz` 200 | 8787 被旧 admin 占用，新进程 bind 失败 | `bash tools/scripts/admin.sh stop` 后重启 |
| 重组后 SPA 404 / 空白 | 未构建 `admin/web/dist` 或 `.env` 仍写 `./web-admin/dist` | `npm run build` + 更新 `WECHATBOT_WEB_ADMIN_DIST_DIR=./admin/web/dist` |
| Windows `cargo build` 拒绝访问 `admin.exe` | admin 进程锁定二进制 | 先 `admin.sh stop` |
| 默认 dist 路径错误 | 代码默认 `{manifest}/admin/web/dist` | 见 `src/admin/server.rs`；可用 env 覆盖 |

### 测试

| 现象 | 原因 | 解法 |
|------|------|------|
| `admin_frontend` 中 SPA 测试 404 | `admin/web/dist` 未构建（`index.html` 不存在） | **先** `cd admin/web && npm run build`，再跑 `test_all.sh` |
| `test_all.sh` 很慢 | 拉 Docker、编译、跑全量集成测试 | 正常，非卡死；需 Docker |
| Playwright e2e 卡住 / preview 不退出 | Playwright `webServer` 内嵌子进程 | 用 `bash tools/scripts/test/run_e2e.sh`（后台 preview + `E2E_SKIP_WEBSERVER=1`） |
| e2e mock 断言 worker 日志失败 | mock 须含 `bot_id`；UI 展示时间线非原始行 | 见 `admin/web/tests/e2e/admin.spec.ts` |

### 安装与发布

| 现象 | 原因 | 解法 |
|------|------|------|
| curl `\| bash install.sh` 404 | 真实脚本在 `tools/scripts/install/` | 根目录保留 **wrapper** `install.sh` |

---

## 路径速查（脚本 / 配置）

| 用途 | 路径 |
|------|------|
| 运维脚本 | `tools/scripts/*.sh`（含 `start_all.sh`/`stop_all.sh`、`start.sh`/`stop.sh`） |
| 公共库 | `tools/scripts/_common.sh` → `PROJECT_ROOT` |
| 后端门禁 | `tools/scripts/test/run_backend_tests.sh` |
| E2E | `tools/scripts/test/run_e2e.sh` |
| Compose 开发 | `deploy/docker-compose.dev.yml` |
| Compose 测试 | `deploy/docker-compose.test.yml` |
| Admin 前端默认 dist | `admin/web/dist`（env: `WECHATBOT_WEB_ADMIN_DIST_DIR`） |
| 安装脚本 | `tools/scripts/install/install.sh`（根 wrapper 转发） |

## 已知踩坑摘要（非重组）

| 现象 | 处理 |
|------|------|
| CI Rust 集成失败 | 本地用 `cargo test --lib`；集成需 PG + Docker |
| Python `login()` 后退出 | 须跑 echo 示例长轮询，见 troubleshooting |
