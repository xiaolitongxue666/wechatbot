# 测试策略

## 协议层独立验证
```bash
bash scripts/dev.sh
```
仅依赖现有 SDK 能力，用于验证扫码登录和基础消息回显。

## 单元测试
```bash
bash scripts/test.sh
```
- 配置解析：模式切换与缺失字段 fail-fast
- 队列：发布/消费、空队列行为
- 媒体：sha256 和存储键生成
- 转发：签名、重试次数、失败路径
- 不依赖外部服务（Postgres/Redis）

## 后端阶段门禁
```bash
bash scripts/test/run_backend_tests.sh
```
- 先执行 `cargo test --lib`
- 再执行 `bash scripts/test_all.sh`
- 只有两者都通过，才进入前端阶段

## 全量集成测试
```bash
bash scripts/test_all.sh
```
自动启动测试容器（pg:5433, redis:6380），建库，编译，运行全部测试，最后清理。需要 Docker。
该脚本覆盖后端测试，不包含 Playwright 前端 E2E。

涵盖：
- 管理后台 API 与 SPA 路由 HTTP 测试
- 数据库仓库 CRUD 测试
- 多会话并发登录与重连
- 消息入库和媒体元数据一致性
- 下游服务异常时重试与最终失败行为

## 前端 E2E（Playwright）
```bash
cd web-admin
bun install
bun run build
bun run test:e2e
```
- 使用 Playwright 校验 Vue 管理端入口和核心交互。
- 建议先执行后端门禁，再执行前端 e2e，避免测试数据不稳定。

## 推荐全链路命令
```bash
bash scripts/test/run_backend_tests.sh
cd web-admin && bun install && bun run build && bun run test:e2e
```
