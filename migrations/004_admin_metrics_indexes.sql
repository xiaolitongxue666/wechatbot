-- =============================================================================
-- Migration 004: Indexes for admin metrics aggregation queries.
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_bot_sessions_bot_id
    ON bot_sessions(bot_id);

CREATE INDEX IF NOT EXISTS idx_chat_messages_received_at
    ON chat_messages(received_at DESC);

CREATE INDEX IF NOT EXISTS idx_forward_events_session_updated_at
    ON forward_events(session_id, updated_at DESC);
