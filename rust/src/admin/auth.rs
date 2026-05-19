use crate::admin::state::AdminState;
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

fn forbidden(message: &str) -> Response {
    (StatusCode::FORBIDDEN, message.to_string()).into_response()
}

fn unauthorized(message: &str) -> Response {
    (StatusCode::UNAUTHORIZED, message.to_string()).into_response()
}

fn parse_bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(AUTHORIZATION)?;
    let text = value.to_str().ok()?;
    let token = text.strip_prefix("Bearer ")?;
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

pub async fn require_permission(
    state: &AdminState,
    headers: &HeaderMap,
    permission: &str,
    bot_id: Option<&str>,
) -> Result<(), Response> {
    let token = if headers.contains_key(AUTHORIZATION) {
        parse_bearer_token(headers).ok_or_else(|| unauthorized("invalid bearer token format"))?
    } else {
        state.default_admin_api_token.clone()
    };

    let principal = state
        .repo
        .resolve_principal_by_token(&token)
        .await
        .map_err(|error| unauthorized(&format!("auth failed: {error}")))?
        .ok_or_else(|| unauthorized("invalid admin token"))?;

    if !principal.permissions.iter().any(|item| item == permission) {
        return Err(forbidden("permission denied"));
    }

    if let Some(target_bot_id) = bot_id {
        let has_scope = principal.bot_scopes.is_empty()
            || principal.bot_scopes.iter().any(|scope_bot_id| scope_bot_id == target_bot_id);
        if !has_scope {
            return Err(forbidden("bot scope denied"));
        }
    }
    Ok(())
}
