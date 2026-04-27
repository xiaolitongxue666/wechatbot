-- =============================================================================
-- Migration 002: Bot-centric architecture, remove tenant/owner.
-- Replaces bot_sessions as primary entity with bots + sessions.
-- =============================================================================

-- Step 1: Create bots table (new primary entity)
CREATE TABLE IF NOT EXISTS bots (
    bot_id      TEXT PRIMARY KEY,
    bot_name    TEXT,
    status      TEXT NOT NULL DEFAULT 'offline',
    credentials_json JSONB,
    last_heartbeat_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Step 2: Migrate existing bot_sessions rows into bots (session_id → bot_id)
INSERT INTO bots (bot_id, status, last_heartbeat_at, created_at, updated_at)
SELECT session_id, status, last_heartbeat_at, created_at, updated_at
FROM bot_sessions
ON CONFLICT (bot_id) DO NOTHING;

-- Step 3: Drop old bot_sessions table and recreate as per-user sessions
DROP TABLE IF EXISTS bot_sessions CASCADE;

CREATE TABLE bot_sessions (
    session_id      TEXT PRIMARY KEY,
    bot_id          TEXT NOT NULL REFERENCES bots(bot_id),
    user_id         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, user_id)
);

-- Step 4: Update chat_messages
-- Drop tenant_id column and update FK
ALTER TABLE chat_messages DROP COLUMN IF EXISTS tenant_id;

-- Re-create index after column change
DROP INDEX IF EXISTS idx_chat_messages_session_time;
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_time
    ON chat_messages(session_id, received_at DESC);

-- Step 5: Update forward_events and forward_dlq (drop tenant-related references if any)
-- forward_events.session_id remains as-is (it will reference a bot_session.session_id)
-- No tenant_id column in these tables currently
