use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::{self, AuthUser, ClientIp, Me, User, Viewer};
use crate::error::{ApiError, ApiResult};
use crate::ratelimit;
use crate::state::AppState;
use acc_core::{handle, limits};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(login))
        .routes(routes!(logout))
        .routes(routes!(me))
        .routes(routes!(verify_email))
        .routes(routes!(resend_verification))
        .routes(routes!(reset_request))
        .routes(routes!(reset_confirm))
}

pub const USER_COLUMNS: &str = "id, handle, email, role, email_verified, suspended, suspend_reason, rating, tier, default_language, handle_changed_at";

pub fn check_password(pw: &str) -> ApiResult<()> {
    if pw.chars().count() < limits::MIN_PASSWORD_LEN {
        return Err(ApiError::bad_request("비밀번호는 8자 이상이어야 합니다."));
    }
    if pw.len() > 256 {
        return Err(ApiError::bad_request("비밀번호가 너무 깁니다."));
    }
    Ok(())
}

/// Checks format and that nobody holds or has reserved the handle.
pub async fn check_handle_available(s: &AppState, h: &str) -> ApiResult<()> {
    if !handle::is_valid(h) {
        return Err(ApiError::bad_request(
            "핸들은 영문 소문자, 숫자, 밑줄로 3~20자여야 합니다.",
        ));
    }
    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE handle = $1)
             OR EXISTS (SELECT 1 FROM reserved_handles WHERE handle = $1 AND until > now())",
    )
    .bind(h)
    .fetch_one(&s.db)
    .await?;
    if taken {
        return Err(ApiError::conflict(
            "HANDLE_TAKEN",
            "이미 사용 중이거나 잠긴 핸들입니다.",
        ));
    }
    Ok(())
}

fn normalize_email(e: &str) -> ApiResult<String> {
    let e = e.trim().to_lowercase();
    let ok = e.len() <= 254
        && e.split_once('@')
            .is_some_and(|(l, d)| !l.is_empty() && d.contains('.') && !d.starts_with('.'))
        && !e.contains(char::is_whitespace);
    if ok {
        Ok(e)
    } else {
        Err(ApiError::bad_request(
            "이메일 주소 형식이 올바르지 않습니다.",
        ))
    }
}

async fn send_verification(s: &AppState, user_id: i64, email: &str) -> ApiResult<()> {
    let token = auth::new_token();
    sqlx::query("INSERT INTO email_tokens (id, user_id, kind, expires_at) VALUES ($1, $2, 'VERIFY', now() + interval '24 hours')")
        .bind(auth::token_hash(&token))
        .bind(user_id)
        .execute(&s.db)
        .await?;
    let link = format!("{}/verify-email?token={token}", s.cfg.web_url);
    s.mail
        .send(
            email,
            "[acc] 이메일 인증",
            &format!("아래 링크를 열어 이메일 인증을 마치세요. 링크는 24시간 동안 유효합니다.\n\n{link}\n"),
        )
        .await;
    Ok(())
}

#[derive(Deserialize, ToSchema)]
pub struct SignupBody {
    pub handle: String,
    pub email: String,
    pub password: String,
    /// Must be true: agrees to the terms of service and privacy policy.
    pub agree_terms: bool,
}

#[utoipa::path(post, path = "/auth/signup", tag = "auth", request_body = SignupBody,
    responses((status = 200, body = Me), (status = 409, body = crate::error::ErrorBody)))]
async fn signup(
    State(s): State<AppState>,
    Json(b): Json<SignupBody>,
) -> ApiResult<(CookieJar, Json<Me>)> {
    if !b.agree_terms {
        return Err(ApiError::bad_request(
            "이용약관과 개인정보처리방침에 동의해야 합니다.",
        ));
    }
    let h = b.handle.trim().to_string();
    check_handle_available(&s, &h).await?;
    let email = normalize_email(&b.email)?;
    check_password(&b.password)?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(&email)
        .fetch_one(&s.db)
        .await?;
    if exists {
        return Err(ApiError::conflict(
            "EMAIL_TAKEN",
            "이미 가입된 이메일입니다.",
        ));
    }
    let pw_hash = auth::hash_password(b.password).await?;
    let role = if s.cfg.admin_handles.iter().any(|a| a == &h) {
        "ADMIN"
    } else {
        "USER"
    };
    let user = sqlx::query_as::<_, User>(&format!(
        "INSERT INTO users (handle, email, pw_hash, role) VALUES ($1, $2, $3, $4) RETURNING {USER_COLUMNS}"
    ))
    .bind(&h)
    .bind(&email)
    .bind(pw_hash)
    .bind(role)
    .fetch_one(&s.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(d) if d.is_unique_violation() => {
            ApiError::conflict("HANDLE_TAKEN", "이미 사용 중인 핸들 또는 이메일입니다.")
        }
        e => e.into(),
    })?;
    send_verification(&s, user.id, &email).await?;
    let cookie = auth::create_session(&s, user.id).await?;
    Ok((CookieJar::new().add(cookie), Json(Me::from(&user))))
}

#[derive(Deserialize, ToSchema)]
pub struct LoginBody {
    /// Handle or email.
    pub login: String,
    pub password: String,
}

#[utoipa::path(post, path = "/auth/login", tag = "auth", request_body = LoginBody,
    responses((status = 200, body = Me), (status = 401, body = crate::error::ErrorBody), (status = 429, body = crate::error::ErrorBody)))]
async fn login(
    State(s): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(b): Json<LoginBody>,
) -> ApiResult<(CookieJar, Json<Me>)> {
    ratelimit::check_login(&s, &ip).await?;
    let login = b.login.trim().to_lowercase();
    let row: Option<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, pw_hash FROM users WHERE (handle = $1 OR email = $1) AND NOT deleted",
    )
    .bind(&login)
    .fetch_optional(&s.db)
    .await?;
    let ok = match &row {
        Some((_, Some(hash))) => auth::verify_password(b.password, hash.clone()).await,
        _ => false,
    };
    let Some((id, _)) = row.filter(|_| ok) else {
        ratelimit::record_login_failure(&s, &ip).await?;
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "BAD_CREDENTIALS",
            "핸들/이메일 또는 비밀번호가 틀렸습니다.",
        ));
    };
    let user =
        sqlx::query_as::<_, User>(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1"))
            .bind(id)
            .fetch_one(&s.db)
            .await?;
    let cookie = auth::create_session(&s, id).await?;
    Ok((CookieJar::new().add(cookie), Json(Me::from(&user))))
}

#[utoipa::path(post, path = "/auth/logout", tag = "auth", responses((status = 204)))]
async fn logout(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<(CookieJar, StatusCode)> {
    if let Some(sid) = auth::current_session_id(&headers) {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(sid)
            .execute(&s.db)
            .await?;
    }
    Ok((
        CookieJar::new().add(auth::clear_cookie(&s)),
        StatusCode::NO_CONTENT,
    ))
}

#[utoipa::path(get, path = "/me", tag = "auth", responses((status = 200, body = Me), (status = 401, body = crate::error::ErrorBody)))]
async fn me(Viewer(v): Viewer) -> ApiResult<Json<Me>> {
    v.map(|u| Json(Me::from(&u)))
        .ok_or_else(ApiError::unauthorized)
}

#[derive(Deserialize, ToSchema)]
pub struct TokenBody {
    pub token: String,
}

#[utoipa::path(post, path = "/auth/verify-email", tag = "auth", request_body = TokenBody, responses((status = 204), (status = 400, body = crate::error::ErrorBody)))]
async fn verify_email(
    State(s): State<AppState>,
    Json(b): Json<TokenBody>,
) -> ApiResult<StatusCode> {
    let uid: Option<i64> = sqlx::query_scalar(
        "UPDATE email_tokens SET used_at = now()
         WHERE id = $1 AND kind = 'VERIFY' AND used_at IS NULL AND expires_at > now() RETURNING user_id",
    )
    .bind(auth::token_hash(b.token.trim()))
    .fetch_optional(&s.db)
    .await?;
    let uid =
        uid.ok_or_else(|| ApiError::bad_request("만료되었거나 이미 사용한 인증 링크입니다."))?;
    sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
        .bind(uid)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/auth/resend-verification", tag = "auth", responses((status = 204), (status = 429, body = crate::error::ErrorBody)))]
async fn resend_verification(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
) -> ApiResult<StatusCode> {
    if u.email_verified {
        return Ok(StatusCode::NO_CONTENT);
    }
    if !ratelimit::once_per(&s, &format!("verify:{}", u.id), 60).await? {
        return Err(ApiError::too_many("1분 후에 다시 보낼 수 있습니다."));
    }
    send_verification(&s, u.id, u.email.as_deref().unwrap_or_default()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct ResetRequestBody {
    pub email: String,
}

/// Always answers 204 so it cannot be used to probe for accounts.
#[utoipa::path(post, path = "/auth/password-reset/request", tag = "auth", request_body = ResetRequestBody, responses((status = 204)))]
async fn reset_request(
    State(s): State<AppState>,
    Json(b): Json<ResetRequestBody>,
) -> ApiResult<StatusCode> {
    let Ok(email) = normalize_email(&b.email) else {
        return Ok(StatusCode::NO_CONTENT);
    };
    if !ratelimit::once_per(&s, &format!("reset:{email}"), 60).await? {
        return Ok(StatusCode::NO_CONTENT);
    }
    let uid: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE email = $1 AND NOT deleted")
            .bind(&email)
            .fetch_optional(&s.db)
            .await?;
    if let Some(uid) = uid {
        let token = auth::new_token();
        sqlx::query("INSERT INTO email_tokens (id, user_id, kind, expires_at) VALUES ($1, $2, 'RESET', now() + interval '30 minutes')")
            .bind(auth::token_hash(&token))
            .bind(uid)
            .execute(&s.db)
            .await?;
        let link = format!("{}/reset-password?token={token}", s.cfg.web_url);
        s.mail
            .send(
                &email,
                "[acc] 비밀번호 재설정",
                &format!("아래 링크에서 새 비밀번호를 정하세요. 링크는 30분 동안 한 번만 쓸 수 있습니다.\n\n{link}\n\n요청한 적이 없다면 이 메일을 무시하세요.\n"),
            )
            .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct ResetConfirmBody {
    pub token: String,
    pub password: String,
}

#[utoipa::path(post, path = "/auth/password-reset/confirm", tag = "auth", request_body = ResetConfirmBody,
    responses((status = 204), (status = 400, body = crate::error::ErrorBody)))]
async fn reset_confirm(
    State(s): State<AppState>,
    Json(b): Json<ResetConfirmBody>,
) -> ApiResult<StatusCode> {
    check_password(&b.password)?;
    let uid: Option<i64> = sqlx::query_scalar(
        "UPDATE email_tokens SET used_at = now()
         WHERE id = $1 AND kind = 'RESET' AND used_at IS NULL AND expires_at > now() RETURNING user_id",
    )
    .bind(auth::token_hash(b.token.trim()))
    .fetch_optional(&s.db)
    .await?;
    let uid = uid.ok_or_else(|| ApiError::bad_request("만료되었거나 이미 사용한 링크입니다."))?;
    let hash = auth::hash_password(b.password).await?;
    let mut tx = s.db.begin().await?;
    sqlx::query("UPDATE users SET pw_hash = $2 WHERE id = $1")
        .bind(uid)
        .bind(hash)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
