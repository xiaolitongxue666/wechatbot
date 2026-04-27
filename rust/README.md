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

```rust
use wechatbot::{WeChatBot, BotOptions};

#[tokio::main]
async fn main() {
    let bot = WeChatBot::new(BotOptions::default());
    let creds = bot.login(false).await.unwrap();
    println!("Logged in: {}", creds.account_id);

    bot.on_message(Box::new(|msg| {
        println!("{}: {}", msg.user_id, msg.text);
    })).await;

    bot.run().await.unwrap();
}
```

## Architecture

```
src/
├── lib.rs           ← Public re-exports
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

The admin dashboard provides a web UI for monitoring and managing bots.

- Start with `bash scripts/start.sh` (one-click) or manually via `cargo run --bin admin`
- Access at `http://127.0.0.1:8787/admin` after startup

### Pages

| Route | Description |
|---|---|
| `/admin` | Overview: total bots, online count, heartbeats, messages, DLQ |
| `/admin/bots` | Bot list with status and quick actions |
| `/admin/bots/{id}` | Bot detail with Start/Stop buttons and QR login code |
| `/admin/bots/{id}/history` | Paginated conversation history (30/page) |
| `/admin/api/overview` | JSON API for dashboard metrics |

### Bot Lifecycle via Admin

1. Open `http://127.0.0.1:8787/admin/bots` → click **New Bot**
2. Fill in tenant_id, owner_id, session_id → submit
3. Page auto-refreshes until QR code appears
4. Scan QR code with WeChat to complete login
5. Start/Stop the bot via buttons on detail page

## Script System

All scripts are under `rust/scripts/`. They source `_common.sh` for shared utilities.

### Quick Reference

| Script | Purpose |
|---|---|
| `start.sh` | **One-click:** services up → migrate → seed → admin start |
| `test.sh` | Run unit tests (no external dependencies) |
| `test_all.sh` | **Full test:** test containers up → migrate → build → test → cleanup |
| `services.sh {up\|down\|status\|restart}` | Manage Docker containers (pg, redis, minio) |
| `db.sh {migrate\|seed\|clear\|reset\|status}` | Database schema and data management |
| `admin.sh {start\|stop\|logs}` | Admin server process lifecycle |
| `dev.sh` | Run echo_bot for protocol-level verification |
| `clean.sh [--all]` | Stop containers, optionally remove volumes and artifacts |
| `status.sh` | Show component health (Docker, DB, Redis, Admin) |

### Common Workflows

**Start development environment (recommended):**
```bash
bash scripts/start.sh
# → Starts Docker services, runs migrations, seeds test data, launches admin
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
bash scripts/db.sh seed             # insert sample data (5 bots, 30 msgs)
bash scripts/db.sh status           # show row counts per table
bash scripts/db.sh reset            # clear data + recreate schema (with confirmation)
bash scripts/admin.sh start         # start admin in background
bash scripts/admin.sh stop          # stop admin
bash scripts/admin.sh logs          # tail admin logs
```

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

# Raw cargo test
cargo test
```

## Documentation

See deployment and operations docs under `rust/doc/`.

## License

MIT
