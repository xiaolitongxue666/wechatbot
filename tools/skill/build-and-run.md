# Build And Run Skill

## 目标

快速完成 Rust SDK 的本地环境搭建与管理后台启动。

## 两种启动模式

| 模式 | 何时用 | 命令 |
|------|--------|------|
| **dev** | 本地测试、Admin 演示、需要 bot-001 等 mock 数据 | `bash tools/scripts/start.sh` |
| **deploy** | 生产/预发、空库启动 | `bash tools/scripts/start.sh --deploy` |

`--no-seed` 与 `--deploy` 等价；`--dev` / `--with-seed` 显式开启 mock。

**禁止**在部署库上执行 `bash tools/scripts/db.sh seed`。

## 操作步骤（dev）

1. 在仓库根目录执行（无需 `cd rust`）
2. `bash tools/scripts/start.sh`
3. 访问：`http://127.0.0.1:8787/admin`
4. 验证：`bash tools/scripts/status.sh`

## 操作步骤（deploy）

1. `cp .env.example .env` 并配置生产连接串
2. `bash tools/scripts/services.sh up`（或外部 PG/Redis）
3. `bash tools/scripts/start.sh --deploy`
4. 在 Admin UI **创建真实 Bot**，勿依赖 mock 数据

## 可选参数

- `--no-admin`：只准备容器与数据库
- `--with-worker`：同时启动 forwarder worker
- 全栈：`bash tools/scripts/start_all.sh [--dev|--deploy]`
- 仅协议验证：`bash tools/scripts/dev.sh`

## 失败排查

- Docker 未启动：先启动 Docker Desktop。
- Rust 工具链缺失：安装 `rustup` 并确认 `cargo --version`。
- 数据库未就绪：重跑 `bash tools/scripts/status.sh` 检查容器健康状态。
