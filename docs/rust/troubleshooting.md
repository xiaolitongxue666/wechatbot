# 排障手册

合并自历史 `trouble_shot.md`、2026-05 仓库重组与测试实践。Agent 速查表见 [agent-memory.md](agent-memory.md#问题与解法总表2026-05-重组--测试)。

## 2026-05 仓库重组

### 症状：按旧文档操作失败

| 旧路径/命令 | 现路径/命令 |
|-------------|-------------|
| `bash scripts/start.sh` | `bash tools/scripts/start.sh` |
| `cd web-admin` | `cd admin/web` |
| `doc/troubleshooting.md` | `docs/rust/troubleshooting.md` |
| `docker-compose.dev.yml`（根目录） | `deploy/docker-compose.dev.yml` |
| `wechat_official/` | 已删除；见 [wechat-official-notes.md](wechat-official-notes.md) |
| `cd rust && cargo build` | 仓库根 `cargo build`（`rust/` 已移除） |

### 脚本路径：`PROJECT_ROOT` 算错

- `tools/scripts/_common.sh`：`PROJECT_ROOT` = 该文件所在目录 **上两级**（`tools/scripts` → 仓库根）。
- `tools/scripts/test/*.sh`：所在目录 **上三级** 才是仓库根，再 `source .../tools/scripts/_common.sh`。
- 若移动脚本目录，必须同步修改上述推导，否则 compose、dist、migrations 路径全部错误。

### 重组后前端或 Admin 不可用

1. 确认 `.env`：`WECHATBOT_WEB_ADMIN_DIST_DIR=./admin/web/dist`
2. `cd admin/web && npm run build`
3. 仓库根：`cargo build --bin admin && bash tools/scripts/admin.sh start`

---

## 管理后台无法访问

### `/admin` 返回 404，但 `/healthz` 为 200

**原因：** 8787 端口被旧进程占用；新 admin 启动时 `bind` 失败（Windows 常见错误 10048），healthz 仍由旧进程响应，但 SPA 路由未正确挂载。

**处理：**

1. 查占用：`netstat -ano | findstr :8787`（Windows）或 `ss -tlnp | grep 8787`
2. 结束旧进程后重启
3. 仓库根目录：`cargo build --bin admin && bash tools/scripts/admin.sh start`
4. 查看仓库根 `.admin.log` 确认有 `admin listening on http://127.0.0.1:8787`

### 访问 `/admin/` 带尾部斜杠返回 404

Axum `ServeDir` 对 **`/admin`（无尾部斜杠）** 返回 SPA；`/admin/` 可能 404。请使用 http://127.0.0.1:8787/admin

### `/admin` 空白或无法加载资源

**原因：** 未构建前端，或 `WECHATBOT_WEB_ADMIN_DIST_DIR` 指向错误路径（常见：仍写 `./web-admin/dist`）。

**处理：**

```bash
cd admin/web && npm run build
# 确认 .env 中 WECHATBOT_WEB_ADMIN_DIST_DIR=./admin/web/dist
```

### Windows：`cargo build` 报「拒绝访问 admin.exe」

**原因：** admin 进程仍在运行，锁定 `target/debug/admin.exe`。

**处理：** `bash tools/scripts/admin.sh stop` 后再 `cargo build`。

---

## 测试

### 集成测试 `admin_frontend`：SPA 用例 404

**现象：** `admin_root_serves_spa_shell`、`spa_history_fallback_route_works` 失败，HTTP 404。

**原因：** 测试走真实 Axum 路由，默认读取 `admin/web/dist`；若未构建则 SPA 不存在。

**处理：**

```bash
cd admin/web && npm run build
bash tools/scripts/test_all.sh
# 或全链路
bash tools/scripts/test/run_backend_tests.sh
```

### `cargo test` 集成测试失败

**现象：** `WECHATBOT_TEST_DATABASE_URL is required`

**处理：**

```bash
export WECHATBOT_TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/wechatbot
cargo test --test admin_frontend
```

日常 CI / 快速门禁使用：`cargo test --lib`（不依赖 Docker）。

### `test_all.sh` 运行很久

**原因：** 启动 Docker 测试栈、迁移、全量 `cargo test`（含集成），正常需数分钟，**不是卡死**。

**处理：** 确保 Docker 可用；跑前已 `npm run build` admin/web。

### Playwright e2e 卡住 / 不退出

**原因：** Playwright 配置内 `webServer` 启动 `vite preview` 子进程，在 Windows 上偶发无法回收。

**处理（推荐）：**

```bash
bash tools/scripts/test/run_e2e.sh
```

脚本在**后台**启动 preview（端口 4174，最多等 30s），设置 `E2E_SKIP_WEBSERVER=1` 跑 Playwright，**退出时 kill 本次 preview**。日志：`.e2e-preview.log`。

### Playwright e2e：`worker log line` 不可见

**原因：** mock 日志需包含 `bot_id` 才会被 `filterWorkerLogLines` 保留；UI 以时间线展示，非原始日志行。

**处理：** 参考 [`../../admin/web/tests/e2e/admin.spec.ts`](../../admin/web/tests/e2e/admin.spec.ts) 中的 mock 格式。

### 推荐全链路测试顺序

```bash
cd admin/web && npm run build
bash tools/scripts/test/run_backend_tests.sh
bash tools/scripts/test/run_e2e.sh
```

---

## 发布与 CI 路径

| 包 | 目录 |
|----|------|
| `@wechatbot/wechatbot` | `reference-sdks/nodejs/` |
| `wechatbot-sdk` | `reference-sdks/python/` |
| `@wechatbot/pi-agent` | `legacy/pi-agent/` |
| Rust 主工程 | 仓库根 |

---

## Python 参考 SDK

### `uv run python -c` 报错

`-c` 后须跟完整代码；`async def` 勿用分号拼单行。改用 heredoc：`uv run python - <<'PY' ... PY`

### 登录后进程退出

仅 `login()` 不会长轮询。使用：`uv run python examples/echo_bot.py`（在 `reference-sdks/python/`）。

---

## Rust 协议与示例

### Echo 不回消息 / 每次都要扫码

- 示例需 Echo 逻辑：见 `examples/echo_bot.rs`
- 强制重扫：`FORCE_QR=1`；默认复用 `~/.wechatbot/credentials.json`

### JSON 反序列化错误（整数枚举、`ret` 缺失等）

已用 `serde_repr`、可选 `ret`、宽松 `ref_msg` 等处理；见 `cargo test --lib`。

### Windows `LNK1104` 链接失败

旧 `echo_bot.exe` 仍在运行，先结束进程再编译。
