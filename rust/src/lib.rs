//! # wechatbot
//!
//! WeChat iLink Bot SDK for Rust.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use wechatbot::{WeChatBot, BotOptions};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bot = WeChatBot::new(BotOptions::default());
//!     bot.login(false).await.unwrap();
//!
//!     bot.on_message(Box::new(|msg| {
//!         println!("{}: {}", msg.user_id, msg.text);
//!     })).await;
//!
//!     bot.run().await.unwrap();
//! }
//! ```

pub mod admin;
pub mod bot;
pub mod config;
pub mod crypto;
pub mod error;
pub mod forwarder;
pub mod ingest;
pub mod protocol;
pub mod queue;
pub mod runtime;
pub mod session;
pub mod storage;
pub mod types;

pub use admin::{admin_router, admin_router_with_runtime, run_admin_repository_pool, run_admin_server};
pub use bot::{BotOptions, MessageHandler, SendContent, WeChatBot};
pub use config::{AdminConfig, AppConfig, DatabaseMode, RuntimeConfig};
pub use crypto::{decrypt_aes_ecb, decrypt_aes_ecb as download_decrypt, encode_aes_key_base64, encode_aes_key_hex, encrypt_aes_ecb, generate_aes_key, decode_aes_key};
pub use error::{Result, WeChatBotError};
pub use forwarder::{ForwardEvent, ForwarderWorker};
pub use ingest::{EventEnvelope, MessageIngestor};
pub use runtime::MultiBotRuntime;
pub use session::{BotSession, BotSessionManager, SessionStatus};
pub use types::*;
