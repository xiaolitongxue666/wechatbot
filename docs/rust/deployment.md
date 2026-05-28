# 部署与启动

## 两种启动模式

| 模式 | 用途 | 数据库 | 命令 |
|------|------|--------|------|
| **dev**（默认） | 本地测试、Admin 演示、联调 | `migrate` + **mock 种子**（bot-001..006、会话、消息） | `bash tools/scripts/start.sh` 或 `start.sh --dev` |
| **deploy** | 生产/预发、真实环境 | **仅** `migrate`，不灌 mock | `bash tools/scripts/start.sh --deploy` |

Mock 数据由 `tools/scripts/db.sh seed` 写入，与 `tests/common/fixtures.rs` 中 `seed_medium_dataset()` 一致。**部署环境禁止执行 `db.sh seed`**。

全栈（admin + worker）同样区分模式：

```bash
bash tools/scripts/start_all.sh          # dev，带 mock
bash tools/scripts/start_all.sh --deploy # 部署，不带 mock
```

手动分步（部署推荐）：

```bash
bash tools/scripts/services.sh up
bash tools/scripts/db.sh migrate          # 不要 seed
bash tools/scripts/admin.sh start
bash tools/scripts/worker.sh start        # 可选
```

---

## Bash 脚本入口

所有脚本位于 `tools/scripts/`，公共库在 `_common.sh`。

| 脚本 | 用途 |
|---|---|
| `start_all.sh [--dev\|--deploy]` | 全栈：容器↑ → 迁移 → [可选 seed] → admin + worker |
| `start.sh [--dev\|--deploy]` | 一键启动：容器↑ → 迁移 → [可选 seed] → 管理后台 |
| `worker.sh {start\|stop\|logs}` | Worker 进程生命周期 |
| `services.sh {up\|down\|status\|restart}` | 管理 Docker 后台容器 (pg, redis, minio) |
| `db.sh {migrate\|seed\|clear\|reset\|status}` | 数据库 schema 和数据管理 |
| `admin.sh {start\|stop\|logs}` | 管理后台进程生命周期 |
| `dev.sh` | echo_bot 协议回环验证 |
| `clean.sh [--all]` | 停止容器，可选清理数据卷和编译产物 |
| `status.sh` | 全局状态检查 |

## 本地部署（dev — 带 mock）

```bash
bash tools/scripts/start.sh
# 或
bash tools/scripts/start_all.sh
```

## 本地/远程部署（deploy — 不带 mock）

1. 启动 Postgres / Redis（及可选 MinIO）：
   ```bash
   bash tools/scripts/services.sh up
   ```
2. 确认运行时依赖：Rust、Docker、Bun（构建 `admin/web`）
3. 配置 `config/app.toml`（`mode=local` / `container` / `remote`）
4. **仅迁移，不 seed**：
   ```bash
   bash tools/scripts/db.sh migrate
   ```
5. 启动服务：
   ```bash
   bash tools/scripts/start.sh --deploy
   # 或分步：
   bash tools/scripts/admin.sh start
   bash tools/scripts/worker.sh start   # 可选
   ```
6. 访问：`http://127.0.0.1:8787/admin`

## 本地部署（分步说明）

以下为 dev / deploy 通用分步；是否 seed 见上文两种模式。

## 容器部署

1. 在容器编排中提供 `postgres`、`redis` 服务名。
2. 配置 `mode=container`。
3. 在应用容器内执行脚本启动。

## 远程部署

1. 配置 `mode=remote` 与远程 URL。
2. 使用安全网络策略和 TLS。
3. 建议通过环境变量注入密钥与敏感配置。
