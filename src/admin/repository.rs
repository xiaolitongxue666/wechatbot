use crate::error::{Result, WeChatBotError};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};


pub(crate) fn paging_limit_offset(page: u64, page_size: u64) -> (i64, i64) {
    let page_size = page_size.clamp(1, 200);
    let page = page.max(1);
    let offset = (page - 1) * page_size;
    (page_size as i64, offset as i64)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BotRow {
    pub bot_id: String,
    #[allow(dead_code)]
    pub bot_name: Option<String>,
    pub status: String,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub messages_today: i64,
    pub forward_failures_today: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BotSessionRow {
    pub session_id: String,
    pub bot_id: String,
    pub user_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct AdminOverview {
    pub total_bots: i64,
    pub online_bots: i64,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub messages_today: i64,
    pub forward_failures_today: i64,
}

#[derive(Debug, Clone)]
pub struct AdminPrincipal {
    pub user_id: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub bot_scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BotForwardPolicy {
    pub bot_id: String,
    pub forwarding_enabled: bool,
    pub allowed_targets: Vec<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatMessageRow {
    #[allow(dead_code)]
    pub message_id: String,
    #[allow(dead_code)]
    pub event_id: String,
    #[allow(dead_code)]
    pub session_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub content_type: String,
    pub text_content: String,
    pub received_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AdminRepository {
    pool: PgPool,
    online_heartbeat_secs: i64,
}

impl AdminRepository {
    pub fn new(pool: PgPool, online_heartbeat_secs: i64) -> Self {
        Self {
            pool,
            online_heartbeat_secs,
        }
    }

    pub async fn overview(&self) -> Result<AdminOverview> {
        let row: (i64, i64, Option<DateTime<Utc>>, i64, i64) = sqlx::query_as(
            r#"
            SELECT
              (SELECT COUNT(*)::bigint FROM bots) AS total_bots,
              (SELECT COUNT(*)::bigint FROM bots
               WHERE LOWER(status) = 'online'
                  OR (last_heartbeat_at IS NOT NULL
                      AND last_heartbeat_at > NOW() - ($1::bigint * INTERVAL '1 second'))) AS online_bots,
              (SELECT MAX(last_heartbeat_at) FROM bots) AS last_hb,
              (SELECT COUNT(*)::bigint FROM chat_messages
               WHERE received_at >= date_trunc('day', NOW())) AS messages_today,
              (SELECT COUNT(*)::bigint
               FROM forward_events fe
               JOIN bot_sessions bs ON bs.session_id = fe.session_id
               WHERE fe.updated_at >= date_trunc('day', NOW())
                 AND fe.status IS DISTINCT FROM 'success') AS fwd_bad_today
            "#,
        )
        .bind(self.online_heartbeat_secs)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("admin overview query failed: {e}")))?;

        Ok(AdminOverview {
            total_bots: row.0,
            online_bots: row.1,
            last_heartbeat_at: row.2,
            messages_today: row.3,
            forward_failures_today: row.4,
        })
    }

    pub async fn list_bots(&self) -> Result<Vec<BotRow>> {
        let rows = sqlx::query_as::<_, BotRow>(
            r#"
            SELECT
              b.bot_id,
              b.bot_name,
              b.status,
              b.last_heartbeat_at,
              COALESCE(message_counts.messages_today, 0)::bigint AS messages_today,
              COALESCE(forward_counts.forward_failures_today, 0)::bigint AS forward_failures_today,
              b.created_at,
              b.updated_at
            FROM bots b
            LEFT JOIN (
              SELECT
                bs.bot_id,
                COUNT(*)::bigint AS messages_today
              FROM chat_messages cm
              JOIN bot_sessions bs ON bs.session_id = cm.session_id
              WHERE cm.received_at >= date_trunc('day', NOW())
              GROUP BY bs.bot_id
            ) AS message_counts ON message_counts.bot_id = b.bot_id
            LEFT JOIN (
              SELECT
                bs.bot_id,
                COUNT(*)::bigint AS forward_failures_today
              FROM forward_events fe
              JOIN bot_sessions bs ON bs.session_id = fe.session_id
              WHERE fe.updated_at >= date_trunc('day', NOW())
                AND fe.status IS DISTINCT FROM 'success'
              GROUP BY bs.bot_id
            ) AS forward_counts ON forward_counts.bot_id = b.bot_id
            ORDER BY b.updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("list bots failed: {e}")))?;
        Ok(rows)
    }

    pub async fn get_bot(&self, bot_id: &str) -> Result<Option<BotRow>> {
        let row = sqlx::query_as::<_, BotRow>(
            r#"
            SELECT
              bot_id,
              bot_name,
              status,
              last_heartbeat_at,
              0::bigint AS messages_today,
              0::bigint AS forward_failures_today,
              created_at,
              updated_at
            FROM bots
            WHERE bot_id = $1
            "#,
        )
        .bind(bot_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("get bot failed: {e}")))?;
        Ok(row)
    }

    pub async fn set_bot_status(&self, bot_id: &str, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE bots
            SET status = $2, updated_at = NOW()
            WHERE bot_id = $1
            "#,
        )
        .bind(bot_id)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("set bot status failed: {error}")))?;
        Ok(())
    }

    pub async fn list_sessions(&self, bot_id: &str) -> Result<Vec<BotSessionRow>> {
        let rows = sqlx::query_as::<_, BotSessionRow>(
            r#"
            SELECT session_id, bot_id, user_id, status, created_at, updated_at
            FROM bot_sessions
            WHERE bot_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(bot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("list bot_sessions failed: {e}")))?;
        Ok(rows)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<BotSessionRow>> {
        let row = sqlx::query_as::<_, BotSessionRow>(
            r#"
            SELECT session_id, bot_id, user_id, status, created_at, updated_at
            FROM bot_sessions
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("get bot_session failed: {e}")))?;
        Ok(row)
    }

    pub async fn delete_bot_hard(&self, bot_id: &str) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| WeChatBotError::Other(format!("begin transaction failed: {error}")))?;

        let session_rows = sqlx::query("SELECT session_id FROM bot_sessions WHERE bot_id = $1")
            .bind(bot_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|error| WeChatBotError::Other(format!("query bot sessions failed: {error}")))?;

        for row in session_rows {
            let session_id: String = row
                .try_get("session_id")
                .map_err(|error| WeChatBotError::Other(format!("read session_id failed: {error}")))?;

            sqlx::query(
                r#"
                DELETE FROM chat_media
                WHERE message_id IN (
                    SELECT message_id FROM chat_messages WHERE session_id = $1
                )
                "#,
            )
            .bind(&session_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| WeChatBotError::Other(format!("delete chat_media failed: {error}")))?;

            sqlx::query("DELETE FROM chat_messages WHERE session_id = $1")
                .bind(&session_id)
                .execute(&mut *tx)
                .await
                .map_err(|error| WeChatBotError::Other(format!("delete chat_messages failed: {error}")))?;

            sqlx::query("DELETE FROM forward_events WHERE session_id = $1")
                .bind(&session_id)
                .execute(&mut *tx)
                .await
                .map_err(|error| WeChatBotError::Other(format!("delete forward_events failed: {error}")))?;

            sqlx::query("DELETE FROM forward_dlq WHERE session_id = $1")
                .bind(&session_id)
                .execute(&mut *tx)
                .await
                .map_err(|error| WeChatBotError::Other(format!("delete forward_dlq failed: {error}")))?;
        }

        sqlx::query("DELETE FROM bot_sessions WHERE bot_id = $1")
            .bind(bot_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| WeChatBotError::Other(format!("delete bot_sessions failed: {error}")))?;

        sqlx::query("DELETE FROM bots WHERE bot_id = $1")
            .bind(bot_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| WeChatBotError::Other(format!("delete bots failed: {error}")))?;

        tx.commit()
            .await
            .map_err(|error| WeChatBotError::Other(format!("commit transaction failed: {error}")))?;

        Ok(())
    }

    pub async fn list_messages_page(
        &self,
        session_id: &str,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<ChatMessageRow>, u64)> {
        let (limit, offset) = paging_limit_offset(page, page_size);

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM chat_messages WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("count chat_messages failed: {e}")))?;

        let rows = sqlx::query_as::<_, ChatMessageRow>(
            r#"
            SELECT message_id, event_id, session_id, from_user_id, to_user_id,
                   content_type, text_content, received_at
            FROM chat_messages
            WHERE session_id = $1
            ORDER BY received_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(session_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WeChatBotError::Other(format!("list chat_messages failed: {e}")))?;

        Ok((rows, total.0 as u64))
    }

    pub async fn upsert_admin_user(
        &self,
        user_id: &str,
        display_name: &str,
        role: &str,
        api_token: &str,
    ) -> Result<()> {
        let token_hash = token_sha256(api_token);
        sqlx::query(
            r#"
            INSERT INTO admin_users (user_id, display_name, role, api_token_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, TRUE, NOW(), NOW())
            ON CONFLICT (user_id) DO UPDATE
            SET display_name = EXCLUDED.display_name,
                role = EXCLUDED.role,
                api_token_hash = EXCLUDED.api_token_hash,
                is_active = TRUE,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(display_name)
        .bind(role)
        .bind(token_hash)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("upsert admin user failed: {error}")))?;
        Ok(())
    }

    pub async fn resolve_principal_by_token(&self, api_token: &str) -> Result<Option<AdminPrincipal>> {
        let token_hash = token_sha256(api_token);
        let user_row = sqlx::query(
            r#"
            SELECT user_id, role
            FROM admin_users
            WHERE api_token_hash = $1 AND is_active = TRUE
            LIMIT 1
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("query admin user failed: {error}")))?;

        let Some(user_row) = user_row else {
            return Ok(None);
        };
        let user_id: String = user_row
            .try_get("user_id")
            .map_err(|error| WeChatBotError::Other(format!("read user_id failed: {error}")))?;
        let role: String = user_row
            .try_get("role")
            .map_err(|error| WeChatBotError::Other(format!("read role failed: {error}")))?;

        let permissions: Vec<String> = sqlx::query_scalar(
            "SELECT permission FROM admin_permissions WHERE role = $1 ORDER BY permission ASC",
        )
        .bind(&role)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("query permissions failed: {error}")))?;

        let bot_scopes: Vec<String> = sqlx::query_scalar(
            "SELECT bot_id FROM admin_user_bot_scopes WHERE user_id = $1 ORDER BY bot_id ASC",
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("query bot scopes failed: {error}")))?;

        Ok(Some(AdminPrincipal {
            user_id,
            role,
            permissions,
            bot_scopes,
        }))
    }

    pub async fn upsert_forward_policy(
        &self,
        bot_id: &str,
        forwarding_enabled: bool,
        allowed_targets: &[String],
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bot_forward_policies (bot_id, forwarding_enabled, allowed_targets, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (bot_id) DO UPDATE
            SET forwarding_enabled = EXCLUDED.forwarding_enabled,
                allowed_targets = EXCLUDED.allowed_targets,
                updated_at = NOW()
            "#,
        )
        .bind(bot_id)
        .bind(forwarding_enabled)
        .bind(allowed_targets)
        .execute(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("upsert forward policy failed: {error}")))?;
        Ok(())
    }

    pub async fn get_forward_policy(&self, bot_id: &str) -> Result<Option<BotForwardPolicy>> {
        let row = sqlx::query(
            r#"
            SELECT bot_id, forwarding_enabled, allowed_targets
            FROM bot_forward_policies
            WHERE bot_id = $1
            LIMIT 1
            "#,
        )
        .bind(bot_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| WeChatBotError::Other(format!("query forward policy failed: {error}")))?;

        let Some(row) = row else {
            return Ok(None);
        };
        let bot_id: String = row
            .try_get("bot_id")
            .map_err(|error| WeChatBotError::Other(format!("read policy bot_id failed: {error}")))?;
        let forwarding_enabled: bool = row
            .try_get("forwarding_enabled")
            .map_err(|error| WeChatBotError::Other(format!("read forwarding_enabled failed: {error}")))?;
        let allowed_targets: Vec<String> = row
            .try_get("allowed_targets")
            .map_err(|error| WeChatBotError::Other(format!("read allowed_targets failed: {error}")))?;

        Ok(Some(BotForwardPolicy {
            bot_id,
            forwarding_enabled,
            allowed_targets,
        }))
    }
}

fn token_sha256(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::{paging_limit_offset, token_sha256};

    #[test]
    fn paging_clamps_size_and_normalizes_page() {
        assert_eq!(paging_limit_offset(0, 10), (10, 0));
        assert_eq!(paging_limit_offset(1, 10), (10, 0));
        assert_eq!(paging_limit_offset(2, 10), (10, 10));
        assert_eq!(paging_limit_offset(1, 500), (200, 0));
    }

    #[test]
    fn token_hash_is_stable() {
        let first = token_sha256("admin-token");
        let second = token_sha256("admin-token");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }
}
