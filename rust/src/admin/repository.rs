use crate::error::{Result, WeChatBotError};
use chrono::{DateTime, Utc};
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
    pub forward_dlq_count: i64,
    pub forward_not_success_count: i64,
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
        let row: (i64, i64, Option<DateTime<Utc>>, i64, i64, i64) = sqlx::query_as(
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
              (SELECT COUNT(*)::bigint FROM forward_dlq) AS dlq,
              (SELECT COUNT(*)::bigint FROM forward_events WHERE status IS DISTINCT FROM 'success') AS fwd_bad
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
            forward_dlq_count: row.4,
            forward_not_success_count: row.5,
        })
    }

    pub async fn list_bots(&self) -> Result<Vec<BotRow>> {
        let rows = sqlx::query_as::<_, BotRow>(
            r#"
            SELECT bot_id, bot_name, status, last_heartbeat_at, created_at, updated_at
            FROM bots
            ORDER BY updated_at DESC
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
            SELECT bot_id, bot_name, status, last_heartbeat_at, created_at, updated_at
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
}

#[cfg(test)]
mod tests {
    use super::paging_limit_offset;

    #[test]
    fn paging_clamps_size_and_normalizes_page() {
        assert_eq!(paging_limit_offset(0, 10), (10, 0));
        assert_eq!(paging_limit_offset(1, 10), (10, 0));
        assert_eq!(paging_limit_offset(2, 10), (10, 10));
        assert_eq!(paging_limit_offset(1, 500), (200, 0));
    }
}
