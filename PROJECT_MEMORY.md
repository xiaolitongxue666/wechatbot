# Project Memory (Compact)

Persistent facts from session history. Covers architecture decisions, integration points, and operational conventions.

## Architecture

1) Rust 主工程（Cargo.toml 根），微信 iLink Bot SDK + Axum admin server (:8787) + 消息转发流水线
2) 存储：PostgreSQL（业务+会话+RBAC）、Redis（事件队列+心跳）、MinIO/local（媒体）
3) 前端：Vue 3 + TypeScript + Vite (admin/web/)，Axum 挂载 admin/web/dist
4) 启动：`cargo run --bin admin`（管理后台+Bot）、`cargo run --bin worker`（转发消费）
5) 开发依赖：`docker compose -f deploy/docker-compose.dev.yml up -d postgres redis`

## Admin API

6) 默认 token：`dev-admin-token`（config.toml `admin.api_token` 可覆盖），Bearer auth
7) 权限：bot.read / bot.write / bot.start_stop / forward.read / forward.write
8) Bot 生命周期：创建 → 扫码(QR) → 用户发消息激活 session → 可收发
9) Bot 发送消息需先有 context_token（用户先发消息给 Bot 后才能回复）
10) 运行时注册表：`MultiBotRuntime.bot_registry`（仅存已 start 的 bot）

## Skill System

11) `skills/` — Python 模块化扩展体系。每个技能是 `skills/<name>/` 目录，导出 `skill` 实例
12) `skills/base.py`：SkillBase（`run()` 主循环）+ BotClient（封装 `POST /send` 等 API）
13) CLI：`python -m skills.run <name>`，快捷脚本 `tools/scripts/skill.sh`（list / start / start-bg / stop / status）
14) Admin API：`GET /admin/api/skills` 返回技能元信息；`SkillsConfig` 在 `config/app.toml [skills]` 段，`SkillsConfig` 从 `lib.rs` 公开导出
15) `freshrss2wxbot.py` — 兼容入口，委派给 `python -m skills.run freshrss`
16) FreshRSS 聚合 RSS 格式：`/freshrss/i/?c=index&a=rss&token=<token>&user=<user>`
17) `sent_articles.txt` — 运行时文件，自动创建，已加入 `.gitignore`

## VPS RSS Stack (Reference)

18) `~/Code/VPS/RSS/rss` — RSSHub + FreshRSS + Clash(mihomo-aio) + Subconverter 统一栈
19) FreshRSS 容器 :8081，公网 nginx `/freshrss/` 路径反代；RSSHub :1200，对外 `/rss/`
20) 聚合 RSS URL 格式：`/freshrss/i/?c=index&a=rss&token=<token>&user=<user>`（user + token 缺一不可）

## Development Conventions

21) 文档根 `docs/`，`docs/rust/agent-memory.md` 是开发者 Agent 记忆入口
22) `cargo test --lib`（单元测试），`cargo test`（全部需 PG+Redis）
23) 不要长时间前台运行 `test_all.sh` / Playwright webServer — 易超时卡死
24) `tools/scripts/dev.sh` 启动开发，`tools/scripts/skill.sh` 管理技能
25) 变更后重建 admin：`cargo build --bin admin`；启动 admin 后 Bot 需重新扫码登录 + 用户发消息激活
