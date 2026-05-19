# 运行与故障处理

## 关键指标
- 在线会话数
- 每会话心跳延迟
- 入队速率与队列积压长度
- 今日消息总数（overview）
- 今日转发失败总数（overview，`forward_events.status != success`）
- 单 bot 今日消息数与单 bot 今日转发失败数（bots 列表）

## 典型故障
- 登录失效：触发强制重新登录并重建上下文
- Redis 不可用：会话状态无法更新，阻断启动
- Postgres 不可用：消息落库失败，触发错误日志与告警
- 下游服务异常：进入重试，超过阈值进入 DLQ

## 运维动作
- 优先确认配置模式（local/container/remote）与连接串
- 查看最近失败事件并定位上游消息与 session
- 对 DLQ 做人工补偿或回放

## 一键启动与分层脚本
- 全栈一键启动：`bash scripts/start_all.sh`
- 仅启动后台：admin `bash scripts/dev/start_backend.sh`，worker `bash scripts/dev/start_worker.sh`
- 快速状态检查：`bash scripts/status.sh`
- 管理前端由 `web-admin/dist` 提供，入口 `http://127.0.0.1:8787/admin`
- `?lang` / `?theme` 查询参数不再影响页面渲染，统一使用 SPA 入口 `/admin`
- 入口不可达时优先执行：`bash scripts/admin.sh stop && bash scripts/services.sh up && bash scripts/admin.sh start`

## RBAC 与 API 鉴权
- 管理端 API 使用 `Authorization: Bearer <token>`
- 默认 token 通过 `WECHATBOT_ADMIN_API_TOKEN` 配置
- 权限点：`bot.read`、`bot.write`、`bot.start_stop`、`forward.read`、`forward.write`
- Bot 级转发策略通过 `/admin/api/bots/{bot_id}/forward-policy` 管理
- 会话历史接口：`/admin/api/sessions/{session_id}/history`

## 日志与排障
- 统一 tracing 初始化，支持 `RUST_LOG` 覆盖默认级别
- 关键链路日志字段建议关注：`bot_id`、`session_id`、`event_id`
- 管理端日志：`.admin.log`；worker 日志：`.worker.log`
