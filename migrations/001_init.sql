CREATE TABLE IF NOT EXISTS bot_sessions (
  session_id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  wx_user_id TEXT,
  status TEXT NOT NULL,
  token_ref TEXT,
  last_heartbeat_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chat_messages (
  message_id TEXT PRIMARY KEY,
  event_id TEXT NOT NULL UNIQUE,
  session_id TEXT NOT NULL REFERENCES bot_sessions(session_id),
  tenant_id TEXT NOT NULL,
  from_user_id TEXT NOT NULL,
  to_user_id TEXT NOT NULL,
  content_type TEXT NOT NULL,
  text_content TEXT NOT NULL,
  raw_payload_json JSONB NOT NULL,
  received_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_session_time
  ON chat_messages(session_id, received_at DESC);

CREATE TABLE IF NOT EXISTS chat_media (
  media_id TEXT PRIMARY KEY,
  message_id TEXT NOT NULL REFERENCES chat_messages(message_id),
  media_type TEXT NOT NULL,
  storage_backend TEXT NOT NULL,
  storage_key TEXT NOT NULL,
  mime_type TEXT,
  size_bytes BIGINT NOT NULL,
  sha256 TEXT NOT NULL,
  encrypt_meta_json JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_media_message_id
  ON chat_media(message_id);

CREATE TABLE IF NOT EXISTS forward_events (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  target_service TEXT NOT NULL,
  status TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  next_retry_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS forward_dlq (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  payload_json JSONB NOT NULL,
  error_message TEXT NOT NULL,
  failed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
