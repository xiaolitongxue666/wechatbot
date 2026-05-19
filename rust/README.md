# wechatbot — Rust SDK

WeChat iLink Bot SDK for Rust — async, type-safe, zero-copy where possible.

## Install

```toml
[dependencies]
wechatbot = "0.1"
tokio = { version = "1", features = ["full"] }
```

Requires Rust 2021 edition. Built on `tokio` + `reqwest`.

## Quick Start

```bash
# 前置条件：Docker Desktop + Rust toolchain
bash scripts/start_all.sh
```

一条命令完成所有步骤：拉取并启动后台服务（PostgreSQL / Redis / MinIO）→ 数据库迁移 → 灌入种子数据 → 启动管理后台 + 转发 worker。

启动后访问 `http://127.0.0.1:8787/admin` 即可进入 Vue 管理界面。

```bash
# 跳过种子数据（仅建表，不插入示例数据）
bash scripts/start.sh --no-seed

# 不启动管理后台（仅启动服务和初始化数据库）
bash scripts/start.sh --no-admin

# 查看全部可用脚本状态
bash scripts/status.sh
```

## Architecture

```
src/
├── lib.rs           ← Public re-exports
├── core/            ← Protocol core namespace re-exports
├── infra/           ← Infra bootstrap helpers (logging, adapters)
├── types.rs         ← All protocol & public types (serde)
├── error.rs         ← Error hierarchy (thiserror)
├── protocol.rs      ← Raw iLink API calls (reqwest)
├── crypto.rs        ← AES-128-ECB encrypt/decrypt + key encoding
├── bot.rs           ← WeChatBot client (login, run, reply, send)
├── session.rs       ← Multi-bot session manager
├── ingest.rs        ← Event normalization and persistence pipeline
├── queue.rs         ← In-memory / Redis event queue abstraction
├── storage/         ← Postgres, Redis state, media store adapters
├── forwarder.rs     ← Async forwarding worker with retry
└── runtime.rs       ← Runtime composition for multi-bot orchestration
```

## API Reference

### Creating a Bot

```rust
use wechatbot::{WeChatBot, BotOptions};

let bot = WeChatBot::new(BotOptions {
    base_url: None,     // default: ilinkai.weixin.qq.com
    cred_path: None,    // default: ~/.wechatbot/credentials.json
    on_qr_url: Some(Box::new(|url| {
        println!("Scan: {}", url);
    })),
    on_error: Some(Box::new(|err| {
        eprintln!("Error: {}", err);
    })),
});
```

### Authentication

```rust
// Login (skips QR if credentials exist)
let creds = bot.login(false).await?;

// Force re-login
let creds = bot.login(true).await?;

// Credentials struct
println!("Token: {}", creds.token);
println!("Base URL: {}", creds.base_url);
println!("Account: {}", creds.account_id);
println!("User: {}", creds.user_id);
```

### Message Handling

```rust
bot.on_message(Box::new(|msg| {
    match msg.content_type {
        ContentType::Text => println!("Text: {}", msg.text),
        ContentType::Image => {
            for img in &msg.images {
                println!("Image URL: {:?}", img.url);
            }
        }
        ContentType::Voice => {
            for voice in &msg.voices {
                println!("Voice: {:?} ({}ms)", voice.text, voice.duration_ms.unwrap_or(0));
            }
        }
        ContentType::File => {
            for file in &msg.files {
                println!("File: {:?}", file.file_name);
            }
        }
        ContentType::Video => println!("Video received"),
    }

    if let Some(ref quoted) = msg.quoted {
        println!("Quoted: {:?}", quoted.title);
    }
})).await;
```

### Sending Messages

```rust
// Reply to incoming message
bot.reply(&msg, "Echo: hello").await?;

// Send to user (needs prior context_token)
bot.send(user_id, "Hello").await?;

// Typing indicator
bot.send_typing(user_id).await?;
```

### Media Operations

```rust
// Reply with media content
bot.reply_media(&msg, SendContent::Image(png_bytes)).await?;
bot.reply_media(&msg, SendContent::File { data, file_name: "report.pdf".into() }).await?;
bot.reply_media(&msg, SendContent::Video(mp4_bytes)).await?;
```

```rust
// Download media from incoming message (priority: image > file > video > voice)
if let Some(media) = bot.download(&msg).await? {
    println!("Type: {}, Size: {} bytes", media.media_type, media.data.len());
    if let Some(name) = &media.file_name {
        println!("Filename: {}", name);
    }
}

// Download a raw CDN reference directly
let raw = bot.download_raw(&msg.images[0].media.as_ref().unwrap(), None).await?;
```

```rust
// Upload to CDN without sending a message
let result = bot.upload(&file_bytes, user_id, 3).await?;
```

### Lifecycle

```rust
// Start polling (blocks)
bot.run().await?;

// Stop
bot.stop().await;
```

## Error Handling

```rust
use wechatbot::WeChatBotError;

match result {
    Err(WeChatBotError::Api { message, errcode, .. }) => {
        if errcode == -14 {
            // session expired — handled automatically
        }
    }
    Err(WeChatBotError::NoContext(user_id)) => {
        // no context_token for this user yet
    }
    Err(WeChatBotError::Transport(e)) => {
        // network error
    }
    _ => {}
}
```

## AES-128-ECB Crypto

```rust
use wechatbot::{generate_aes_key, encrypt_aes_ecb, decrypt_aes_ecb, decode_aes_key};

// Generate key
let key = generate_aes_key();

// Encrypt/decrypt
let ciphertext = encrypt_aes_ecb(b"Hello", &key);
let plaintext = decrypt_aes_ecb(&ciphertext, &key)?;

// Decode protocol key (handles all 3 formats)
let key = decode_aes_key("ABEiM0RVZneImaq7zN3u/w==")?;
let key = decode_aes_key("00112233445566778899aabbccddeeff")?;
```

## Types

All protocol types derive `Serialize` + `Deserialize` + `Clone` + `Debug`:

```rust
// Wire-level (protocol)
WireMessage, WireMessageItem, CDNMedia, TextItem, ImageItem, ...

// Parsed (user-friendly)
IncomingMessage, ImageContent, VoiceContent, FileContent, VideoContent

// Auth
Credentials

// Enums
MessageType, MessageState, MessageItemType, ContentType, MediaType
```

## Admin Dashboard

The admin dashboard is now a Vue3 SPA served at `/admin`, with Axum exposing JSON APIs at `/admin/api/*`.

- Start with `bash scripts/start.sh` (one-click) or manually via `cargo run --bin admin`
- Access at `http://127.0.0.1:8787/admin` after startup
- `?lang=...&theme=...` query 参数在 Vue SPA 中已不再作为服务端渲染开关，访问入口请直接使用 `/admin`
- 如果无法打开后台，先执行 `bash scripts/status.sh`；必要时执行 `bash scripts/admin.sh stop && bash scripts/admin.sh start`

### Layout

- Top Bar: title + token controls + theme/language switches
- Middle: overview/bot list view and bot detail/action view (switch on bot selection)
- Bottom: dual scrolling log panels (session logs + system logs)

### API Surface

| Route | Description |
|---|---|
| `/admin/api/overview` | Dashboard metrics |
| `/admin/api/bots` | List bots / create bot |
| `/admin/api/bots/{id}` | Bot detail / delete bot |
| `/admin/api/bots/{id}/start` | Start bot |
| `/admin/api/bots/{id}/stop` | Stop bot |
| `/admin/api/bots/{id}/status` | Bot runtime status |
| `/admin/api/bots/{id}/forward-policy` | Read/Update forward policy |
| `/admin/api/sessions/{session_id}/history` | Paginated session history |
| `/admin/api/system-logs/admin` | Admin runtime log tail |
| `/admin/api/system-logs/worker` | Worker runtime log tail |

### Metrics Semantics

- `GET /admin/api/overview` exposes:
  - `messages_today`: 今日消息总数
  - `forward_failures_today`: 今日转发失败总数（`forward_events.status != success`）
- `GET /admin/api/bots` exposes per-bot counters:
  - `messages_today`: 单个 bot 今日消息数
  - `forward_failures_today`: 单个 bot 今日转发失败数（`forward_events.status != success`）

## Script System

All scripts are under `rust/scripts/`. They source `_common.sh` for shared utilities.

### Quick Reference

| Script | Purpose |
|---|---|
| `start_all.sh` | **One-click full stack:** services up → migrate → seed → admin → worker |
| `start.sh` | **One-click:** services up → migrate → seed → admin start |
| `worker.sh {start\|stop\|logs}` | Forwarder worker process lifecycle |
| `test.sh` | Run unit tests (no external dependencies) |
| `test_all.sh` | **Backend full test:** test containers up → migrate → build → test → cleanup |
| `scripts/dev/start_backend.sh` | Start admin backend with pre-checks |
| `scripts/dev/start_worker.sh` | Start worker process with pre-checks |
| `scripts/test/run_backend_tests.sh` | Run backend test gates (`cargo test --lib` + `test_all.sh`) |
| `services.sh {up\|down\|status\|restart}` | Manage Docker containers (pg, redis, minio) |
| `db.sh {migrate\|seed\|clear\|reset\|status}` | Database schema and data management |
| `admin.sh {start\|stop\|logs}` | Admin server process lifecycle |
| `dev.sh` | Run echo_bot for protocol-level verification |
| `clean.sh [--all]` | Stop containers, optionally remove volumes and artifacts |
| `status.sh` | Show component health (Docker, DB, Redis, Admin) |

### Common Workflows

**Start development environment (recommended):**
```bash
bash scripts/start_all.sh
# → Starts Docker services, runs migrations, seeds test data, launches admin and worker
# → Admin: http://127.0.0.1:8787/admin
```

**Start without seeding data:**
```bash
bash scripts/start.sh --no-seed
```

**Run all tests (unit + integration):**
```bash
bash scripts/test_all.sh
# → Spins up test containers, runs full suite, auto-cleanup
```

**Run unit tests only:**
```bash
bash scripts/test.sh
```

**Manage services individually:**
```bash
bash scripts/services.sh up         # start PostgreSQL, Redis, MinIO
bash scripts/services.sh down       # stop (keep volumes)
bash scripts/services.sh status     # show container status
bash scripts/db.sh migrate          # create tables
bash scripts/db.sh seed             # insert sample data (5 bots, 30 msgs, 5 fwd, 2 dlq)
bash scripts/db.sh status           # show row counts per table
bash scripts/db.sh reset            # clear data + recreate schema (with confirmation)
bash scripts/admin.sh start         # start admin in background
bash scripts/admin.sh stop          # stop admin
bash scripts/admin.sh logs          # tail admin logs
bash scripts/worker.sh start        # start worker in background
bash scripts/worker.sh stop         # stop worker
bash scripts/worker.sh logs         # tail worker logs
```

### Development Containers (docker-compose.dev.yml)

The local development stack starts three infrastructure containers:

- `rust-postgres-1` (`postgres:16`, `5432`): primary relational database for bot sessions, messages, and admin queries. Uses the `database.local_url` setting in `config/app.toml` and persists data in the `pgdata` volume.
- `rust-redis-1` (`redis:7`, `6379`): cache/state/queue backend used by Redis-based runtime features. Uses `redis.local_url` in `config/app.toml`.
- `rust-minio-1` (`minio/minio:latest`, `9000` + `9001`): S3-compatible object storage for media workflows. `9000` is the S3 API endpoint and `9001` is the MinIO console. Uses the `miniodata` volume for persistence.

Notes:
- Current default media backend is `localfs` (`media.backend = "localfs"`), so media writes go to local filesystem unless you switch backend mode.
- MinIO is still useful in development because the endpoint and credentials are preconfigured and ready when switching to S3-compatible storage.

**Full cleanup:**
```bash
bash scripts/clean.sh --all
# → Stops containers, removes volumes, deletes build artifacts and logs
```

**Check system status:**
```bash
bash scripts/status.sh
# → Docker containers, database connectivity, Redis, admin server, build state
```

**Echo bot (protocol verification):**
```bash
bash scripts/dev.sh
# → Requires WeChat scan to connect; echoes back all received messages
```

## Testing

```bash
# Unit tests only (fast, no external services)
bash scripts/test.sh

# Full test suite (requires Docker)
bash scripts/test_all.sh

# Backend gate (unit + integration)
bash scripts/test/run_backend_tests.sh

# Frontend E2E
cd web-admin && bun install && bun run build && bun run test:e2e

# Raw cargo test
cargo test
```

## Bun Web Admin

```bash
cd web-admin
bun install
 
# Local development (Vite + proxy to Axum :8787)
bun run dev

# Build for Axum static serving under /admin
bun run build

# E2E
bun run test:e2e
```

- Default static output: `web-admin/dist`
- Admin backend serves built assets at `http://127.0.0.1:8787/admin`
- Frontend API auth uses `Authorization: Bearer <token>`, default token is `dev-admin-token`

## Documentation

See deployment and operations docs under `rust/doc/`.

## License

MIT
