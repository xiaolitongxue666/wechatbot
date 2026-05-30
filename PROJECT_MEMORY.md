# Project Memory (Compact)

Persistent facts from session history. Covers architecture decisions, integration points, and operational conventions.

## Architecture

1) Rust 主工程（Cargo.toml 根），微信 iLink Bot SDK + Axum admin server (:8787) + 消息转发流水线
2) 存储：PostgreSQL（业务+会话+RBAC）、Redis（事件队列+心跳）、MinIO/local（媒体）
3) 前端：Vue 3 + TypeScript + Vite，位于 `admin/web/`，由 Axum 挂载 `admin/web/dist`
4) 启动模式：`cargo run --bin admin`（管理后台+Bot）、`cargo run --bin worker`（转发消费）
5) 开发依赖：`docker compose -f deploy/docker-compose.dev.yml up -d postgres redis`

## Admin API Key Points

6) Admin API 默认 token：`dev-admin-token`（config.toml 中 `admin.api_token` 可覆盖）
7) Auth：Bearer token → `Authorization: Bearer <token>`，权限分 bot.read / bot.write / bot.start_stop / forward.read / forward.write
8) Bot 生命周期：创建 → 扫码(QR) → 用户发送消息激活 session → 可收发
9) Bot 发送消息需先有 context_token（用户先发消息给 Bot 后才能回复）
10) Bot 运行时注册表在 `MultiBotRuntime.bot_registry`（`Arc<RwLock<HashMap<String, Arc<WeChatBot>>>`），只存已 start 的 bot

## RSS → WeChat Pipeline

11) `freshrss2wxbot.py` — 仓库根目录 Python 脚本，轮询 FreshRSS 聚合 RSS，去重记录，推送到微信
12) FreshRSS 实例（VPS）：`xiaolitongxue.com.cn/freshrss`，用户 `leonpa1987@gmail.com`
13) FreshRSS 聚合 RSS URL 格式：`/freshrss/i/?c=index&a=rss&token=<token>&user=<user>`（需同时传 user + token）
14) FreshRSS token 在用户 `config.php` 中设为 `'token' => '<hex>'`，由 `tokenIsOk()` 校验
15) `sent_articles.txt` — 已推送文章 ID 记录，脚本自动维护
16) 推送接口：`POST /admin/api/bots/{bot_id}/send`，body `{"user_id":"...","text":"..."}`，Bearer auth
17) `bot_send_json` 处理器在 `src/admin/handlers/api.rs`，路由在 `server.rs`
18) 推送需 4 个配置：`BOT_ADMIN_URL` / `BOT_ADMIN_TOKEN` / `BOT_ID` / `BOT_USER_ID`

## VPS RSS Stack (Reference)

19) VPS RSS 栈位于 `/Users/liyong/Code/VPS/RSS/rss`，含 RSSHub + FreshRSS + Clash(mihomo-aio) + Subconverter
20) FreshRSS 容器端口 `8081:80`，公网通过 nginx `/freshrss/` 路径反代，rewrite 规则剥前缀
21) RSSHub 容器端口 `1200:1200`，对外 `/rss/` 路径

## Development Conventions

22) 文档根在 `docs/`，`docs/rust/agent-memory.md` 是开发者 Agent 记忆入口
23) 测试命令：`cargo test --lib`（单元测试），`cargo test`（全部，含集成测试需 PG+Redis）
24) 不要长时间前台运行 `test_all.sh` / Playwright webServer — 易超时卡死
25) `tools/scripts/dev.sh` 启动开发环境，`tools/scripts/start.sh` 启动 mock 演示
