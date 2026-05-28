# 测试策略

## 协议层独立验证
```bash
bash tools/scripts/dev.sh
```
仅依赖现有 SDK 能力，用于验证扫码登录和基础消息回显。

## 单元测试
```bash
bash tools/scripts/test.sh
# 或
cargo test --lib
```
- 配置解析：模式切换与缺失字段 fail-fast
- 队列：发布/消费、空队列行为
- 媒体：sha256 和存储键生成
- 转发：签名、重试次数、失败路径
- 不依赖外部服务（Postgres/Redis）

## 后端阶段门禁

```bash
cd admin/web && npm run build   # admin_frontend 集成测试依赖 dist，必须先构建
bash tools/scripts/test/run_backend_tests.sh
```
- 先执行 `cargo test --lib`
- 再执行 `bash tools/scripts/test_all.sh`
- 只有两者都通过，才进入前端阶段

## 全量集成测试
```bash
bash tools/scripts/test_all.sh
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

**推荐**（preview 后台启动，跑完自动清理，避免 Playwright webServer 卡死）：

```bash
bash tools/scripts/test/run_e2e.sh
# 或
cd admin/web && npm run test:e2e
```

脚本行为：
1. 若 `admin/web/dist` 不存在则先 `npm run build`
2. 后台启动 `vite preview`（端口 4174），最多等待 30s
3. `E2E_SKIP_WEBSERVER=1` 下运行 Playwright（不嵌套启动 webServer）
4. 退出时停止本次脚本启动的 preview 进程

日志：仓库根 `.e2e-preview.log`；PID：`.e2e-preview.pid`（gitignore）

直接调 Playwright（需 preview 已在跑）：`cd admin/web && npm run test:e2e:raw`

## 推荐全链路命令

```bash
cd admin/web && npm run build
bash tools/scripts/test/run_backend_tests.sh
bash tools/scripts/test/run_e2e.sh
```
