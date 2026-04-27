# Build And Run Skill

## 目标

快速完成 Rust SDK 的本地环境搭建与管理后台启动。

## 操作步骤

1. 进入目录：`cd rust`
2. 启动全流程：`bash scripts/start.sh`
3. 访问后台：`http://127.0.0.1:8787/admin`
4. 验证状态：`bash scripts/status.sh`

## 可选模式

- 跳过种子：`bash scripts/start.sh --no-seed`
- 不启后台：`bash scripts/start.sh --no-admin`
- 仅协议验证：`bash scripts/dev.sh`

## 失败排查

- Docker 未启动：先启动 Docker Desktop。
- Rust 工具链缺失：安装 `rustup` 并确认 `cargo --version`。
- 数据库未就绪：重跑 `bash scripts/status.sh` 检查容器健康状态。
