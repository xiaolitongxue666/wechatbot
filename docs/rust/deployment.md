# 部署与启动

## Bash 脚本入口

所有脚本位于 `tools/scripts/`，公共库在 `_common.sh`。

| 脚本 | 用途 |
|---|---|
| `start_all.sh` | 一键全栈：容器↑ → 迁移 → 种子 → admin + worker |
| `start.sh` | 一键启动：容器↑ → 迁移 → 种子 → 管理后台 |
| `worker.sh {start\|stop\|logs}` | Worker 进程生命周期 |
| `services.sh {up\|down\|status\|restart}` | 管理 Docker 后台容器 (pg, redis, minio) |
| `db.sh {migrate\|seed\|clear\|reset\|status}` | 数据库 schema 和数据管理 |
| `admin.sh {start\|stop\|logs}` | 管理后台进程生命周期 |
| `dev.sh` | echo_bot 协议回环验证 |
| `clean.sh [--all]` | 停止容器，可选清理数据卷和编译产物 |
| `status.sh` | 全局状态检查 |

## 本地部署

1. 启动本地 Postgres 和 Redis：
   ```bash
   bash tools/scripts/services.sh up
   ```
2. 确认运行时依赖：
   - Rust / Cargo
   - Docker / Docker Compose
   - Bun（用于构建 `admin/web`，`admin.sh` 启动时会自动构建）
3. 配置 `app.toml` 使用 `mode=local`（默认）。
4. 数据库迁移与种子数据：
   ```bash
   bash tools/scripts/db.sh migrate
   bash tools/scripts/db.sh seed    # 可选：插入演示数据
   ```
5. 启动应用（推荐全栈）：
   ```bash
   bash tools/scripts/start_all.sh
   ```
   或按组件启动：
   ```bash
   bash tools/scripts/admin.sh start    # 后台启动管理程序
   bash tools/scripts/worker.sh start   # 后台启动 worker
   # 或仅管理后台一键启动：
   bash tools/scripts/start.sh
   ```
6. 访问管理端：
   - `http://127.0.0.1:8787/admin`
   - 管理端指标口径：
     - 概览：今日消息总数、今日转发失败总数
     - Bot 列表：单 bot 今日消息数、单 bot 今日转发失败数

## 容器部署

1. 在容器编排中提供 `postgres`、`redis` 服务名。
2. 配置 `mode=container`。
3. 在应用容器内执行脚本启动。

## 远程部署

1. 配置 `mode=remote` 与远程 URL。
2. 使用安全网络策略和 TLS。
3. 建议通过环境变量注入密钥与敏感配置。
