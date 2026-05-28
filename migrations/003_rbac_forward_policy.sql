-- =============================================================================
-- Migration 003: admin RBAC + bot forwarding policy
-- =============================================================================

CREATE TABLE IF NOT EXISTS admin_users (
    user_id          TEXT PRIMARY KEY,
    display_name     TEXT NOT NULL,
    role             TEXT NOT NULL,
    api_token_hash   TEXT NOT NULL UNIQUE,
    is_active        BOOLEAN NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS admin_permissions (
    role             TEXT NOT NULL,
    permission       TEXT NOT NULL,
    PRIMARY KEY (role, permission)
);

CREATE TABLE IF NOT EXISTS admin_user_bot_scopes (
    user_id          TEXT NOT NULL REFERENCES admin_users(user_id) ON DELETE CASCADE,
    bot_id           TEXT NOT NULL REFERENCES bots(bot_id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, bot_id)
);

CREATE TABLE IF NOT EXISTS bot_forward_policies (
    bot_id              TEXT PRIMARY KEY REFERENCES bots(bot_id) ON DELETE CASCADE,
    forwarding_enabled  BOOLEAN NOT NULL DEFAULT TRUE,
    allowed_targets     TEXT[] NOT NULL DEFAULT ARRAY['webhook']::TEXT[],
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO admin_permissions (role, permission) VALUES
    ('admin', 'bot.read'),
    ('admin', 'bot.write'),
    ('admin', 'bot.start_stop'),
    ('admin', 'forward.read'),
    ('admin', 'forward.write'),
    ('operator', 'bot.read'),
    ('operator', 'bot.start_stop'),
    ('operator', 'forward.read'),
    ('viewer', 'bot.read'),
    ('viewer', 'forward.read')
ON CONFLICT DO NOTHING;
