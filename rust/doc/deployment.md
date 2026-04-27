# 部署与启动

## Bash 脚本入口
- `rust/scripts/dev.sh`：开发启动
- `rust/scripts/test.sh`：测试执行
- `rust/scripts/run_protocol_echo.sh`：协议回归验证
- `rust/scripts/migrate.sh`：数据库迁移

## 本地部署
1. 启动本地 Postgres 和 Redis。
2. 配置 `app.toml` 使用 `mode=local`。
3. 运行迁移脚本后启动应用。

## 容器部署
1. 在容器编排中提供 `postgres`、`redis` 服务名。
2. 配置 `mode=container`。
3. 在应用容器内执行脚本启动。

## 远程部署
1. 配置 `mode=remote` 与远程 URL。
2. 使用安全网络策略和 TLS。
3. 建议通过环境变量注入密钥与敏感配置。
