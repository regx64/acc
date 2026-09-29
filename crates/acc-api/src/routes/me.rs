//! `/settings`: default language, password, handle, account deletion.

use acc_core::{handle, Language};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::auth::{check_handle_available, check_password};
use crate::auth::{self, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(update_settings))
        .routes(routes!(change_password))
        .routes(routes!(change_handle))
        .routes(routes!(delete_account))
}

#[derive(Deserialize, ToSchema)]
pub struct SettingsBody {
    /// Language id, or null to clear.
    pub default_language: Option<String>,
}

#[utoipa::path(patch, path = "/me/settings", tag = "me", request_body = SettingsBody, responses((status = 204)))]
async fn update_settings(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
    Json(b): Json<SettingsBody>,
) -> ApiResult<StatusCode> {
    if let Some(l) = &b.default_language {
        l.parse::<Language>()
            .map_err(|_| ApiError::bad_request("지원하지 않는 언어입니다."))?;
    }
    sqlx::query("UPDATE users SET default_language = $2 WHERE id = $1")
        .bind(u.id)
        .bind(b.default_language)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn require_password(s: &AppState, user_id: i64, password: String) -> ApiResult<()> {
    let hash: Option<String> = sqlx::query_scalar("SELECT pw_hash FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&s.db)
        .await?;
    let ok = match hash {
        Some(h) => auth::verify_password(password, h).await,
        None => false,
    };
    if ok {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "BAD_PASSWORD",
            "현재 비밀번호가 틀렸습니다.",
        ))
    }
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordBody {
    pub current_password: String,
    pub new_password: String,
}

/// Also signs out every other session.
#[utoipa::path(post, path = "/me/password", tag = "me", request_body = PasswordBody, responses((status = 204), (status = 403, body = crate::error::ErrorBody)))]
async fn change_password(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
    headers: HeaderMap,
    Json(b): Json<PasswordBody>,
) -> ApiResult<StatusCode> {
    check_password(&b.new_password)?;
    require_password(&s, u.id, b.current_password).await?;
    let hash = auth::hash_password(b.new_password).await?;
    let current = auth::current_session_id(&headers).unwrap_or_default();
    let mut tx = s.db.begin().await?;
    sqlx::query("UPDATE users SET pw_hash = $2 WHERE id = $1")
        .bind(u.id)
        .bind(hash)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
        .bind(u.id)
        .bind(current)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct HandleBody {
    pub handle: String,
}

/// Once per 90 days. The old handle is locked for 180 days and redirects.
#[utoipa::path(post, path = "/me/handle", tag = "me", request_body = HandleBody, responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn change_handle(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
    Json(b): Json<HandleBody>,
) -> ApiResult<StatusCode> {
    let new = b.handle.trim().to_string();
    if new == u.handle() {
        return Ok(StatusCode::NO_CONTENT);
    }
    if let Some(at) = u.handle_changed_at {
        let next = at + chrono::Duration::days(handle::CHANGE_INTERVAL_DAYS);
        if next > chrono::Utc::now() {
            return Err(ApiError::conflict(
                "HANDLE_CHANGE_TOO_SOON",
                format!(
                    "핸들은 90일에 한 번 바꿀 수 있습니다. 다음 변경 가능일: {}",
                    next.format("%Y-%m-%d")
                ),
            ));
        }
    }
    check_handle_available(&s, &new).await?;
    let mut tx = s.db.begin().await?;
    sqlx::query("UPDATE users SET handle = $2, handle_changed_at = now() WHERE id = $1")
        .bind(u.id)
        .bind(&new)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(d) if d.is_unique_violation() => {
                ApiError::conflict("HANDLE_TAKEN", "이미 사용 중인 핸들입니다.")
            }
            e => e.into(),
        })?;
    sqlx::query(
        "INSERT INTO reserved_handles (handle, user_id, until) VALUES ($1, $2, now() + make_interval(days => $3))
         ON CONFLICT (handle) DO UPDATE SET user_id = EXCLUDED.user_id, until = EXCLUDED.until",
    )
    .bind(u.handle())
    .bind(u.id)
    .bind(handle::RELEASE_LOCK_DAYS as i32)
    .execute(&mut *tx)
    .await?;
    // A handle this user held earlier now points at nobody else.
    sqlx::query("DELETE FROM reserved_handles WHERE handle = $1 AND user_id = $2")
        .bind(&new)
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteBody {
    pub password: String,
}

/// Personal data is erased at once. Submissions and posts stay, shown as a
/// deleted user; the handle is locked forever.
#[utoipa::path(post, path = "/me/delete", tag = "me", request_body = DeleteBody, responses((status = 204), (status = 403, body = crate::error::ErrorBody)))]
async fn delete_account(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
    Json(b): Json<DeleteBody>,
) -> ApiResult<(CookieJar, StatusCode)> {
    require_password(&s, u.id, b.password).await?;
    let mut tx = s.db.begin().await?;
    sqlx::query(
        "INSERT INTO reserved_handles (handle, user_id, until) VALUES ($1, NULL, 'infinity')
         ON CONFLICT (handle) DO UPDATE SET user_id = NULL, until = 'infinity'",
    )
    .bind(u.handle())
    .execute(&mut *tx)
    .await?;
    // Handles released earlier by this user stop redirecting to the account.
    sqlx::query("UPDATE reserved_handles SET user_id = NULL WHERE user_id = $1")
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE users SET handle = NULL, email = NULL, pw_hash = NULL, deleted = TRUE, email_verified = FALSE,
                rating = 0, tier = 0, default_language = NULL, suspend_reason = NULL
         WHERE id = $1",
    )
    .bind(u.id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM email_tokens WHERE user_id = $1")
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((
        CookieJar::new().add(auth::clear_cookie(&s)),
        StatusCode::NO_CONTENT,
    ))
}
