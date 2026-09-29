//! Sessions, passwords, client IPs and the user extractors.

use std::net::{IpAddr, SocketAddr};

use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use axum::extract::{ConnectInfo, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub const SESSION_COOKIE: &str = "acc_session";
pub const SESSION_DAYS: i64 = 30;

/// The signed-in user as loaded for each request.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub handle: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub suspended: bool,
    pub suspend_reason: Option<String>,
    pub rating: i32,
    pub tier: i16,
    pub default_language: Option<String>,
    pub handle_changed_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "ADMIN"
    }
    pub fn handle(&self) -> &str {
        self.handle.as_deref().unwrap_or("")
    }
}

/// `GET /me` payload.
#[derive(Serialize, ToSchema)]
pub struct Me {
    pub id: i64,
    pub handle: String,
    pub email: String,
    pub role: String,
    pub email_verified: bool,
    pub suspended: bool,
    pub suspend_reason: Option<String>,
    pub rating: i32,
    pub tier: i16,
    pub default_language: Option<String>,
    pub handle_changed_at: Option<DateTime<Utc>>,
}

impl From<&User> for Me {
    fn from(u: &User) -> Self {
        Me {
            id: u.id,
            handle: u.handle().to_string(),
            email: u.email.clone().unwrap_or_default(),
            role: u.role.clone(),
            email_verified: u.email_verified,
            suspended: u.suspended,
            suspend_reason: u.suspend_reason.clone(),
            rating: u.rating,
            tier: u.tier,
            default_language: u.default_language.clone(),
            handle_changed_at: u.handle_changed_at,
        }
    }
}

// ---------- tokens & passwords ----------

/// 32 random bytes, URL-safe base64.
pub fn new_token() -> String {
    let mut buf = [0u8; 32];
    rand::fill(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

pub fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn hash_password(password: String) -> ApiResult<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(ApiError::internal)
    })
    .await
    .map_err(ApiError::internal)?
}

pub async fn verify_password(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|h| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &h)
                    .is_ok()
            })
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

// ---------- sessions ----------

/// Creates a session and returns the cookie that carries it.
pub async fn create_session(state: &AppState, user_id: i64) -> ApiResult<Cookie<'static>> {
    let token = new_token();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, now() + make_interval(days => $3))",
    )
    .bind(token_hash(&token))
    .bind(user_id)
    .bind(SESSION_DAYS as i32)
    .execute(&state.db)
    .await?;
    Ok(session_cookie(
        state,
        token,
        time::Duration::days(SESSION_DAYS),
    ))
}

pub fn session_cookie(state: &AppState, value: String, max_age: time::Duration) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value))
        .http_only(true)
        .secure(state.cfg.secure_cookies)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(max_age)
        .build()
}

pub fn clear_cookie(state: &AppState) -> Cookie<'static> {
    session_cookie(state, String::new(), time::Duration::ZERO)
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|c| Cookie::parse(c.trim().to_string()).ok())
        .find(|c| c.name() == SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .filter(|v| !v.is_empty())
}

/// Hash of the current request's session token, if any.
pub fn current_session_id(headers: &HeaderMap) -> Option<String> {
    cookie_token(headers).map(|t| token_hash(&t))
}

async fn load_viewer(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<User>> {
    let Some(sid) = current_session_id(headers) else {
        return Ok(None);
    };
    let user = sqlx::query_as::<_, User>(
        "SELECT u.id, u.handle, u.email, u.role, u.email_verified, u.suspended, u.suspend_reason,
                u.rating, u.tier, u.default_language, u.handle_changed_at
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.id = $1 AND s.expires_at > now() AND NOT u.deleted",
    )
    .bind(sid)
    .fetch_optional(&state.db)
    .await?;
    Ok(user)
}

// ---------- extractors ----------

/// Signed-in user if there is one.
pub struct Viewer(pub Option<User>);

#[derive(Clone)]
struct CachedViewer(Option<User>);

impl FromRequestParts<AppState> for Viewer {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(c) = parts.extensions.get::<CachedViewer>() {
            return Ok(Viewer(c.0.clone()));
        }
        let user = load_viewer(state, &parts.headers).await?;
        parts.extensions.insert(CachedViewer(user.clone()));
        Ok(Viewer(user))
    }
}

/// Any signed-in user.
pub struct AuthUser(pub User);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match Viewer::from_request_parts(parts, state).await?.0 {
            Some(u) => Ok(AuthUser(u)),
            None => Err(ApiError::unauthorized()),
        }
    }
}

/// Signed-in, email verified and not suspended: may submit and write.
pub struct ActiveUser(pub User);

impl FromRequestParts<AppState> for ActiveUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(u) = AuthUser::from_request_parts(parts, state).await?;
        if !u.email_verified {
            return Err(ApiError::new(
                axum::http::StatusCode::FORBIDDEN,
                "EMAIL_NOT_VERIFIED",
                "이메일 인증 후에 이용할 수 있습니다.",
            ));
        }
        if u.suspended {
            return Err(ApiError::new(
                axum::http::StatusCode::FORBIDDEN,
                "SUSPENDED",
                format!(
                    "정지된 계정입니다. 사유: {}",
                    u.suspend_reason.as_deref().unwrap_or("없음")
                ),
            ));
        }
        Ok(ActiveUser(u))
    }
}

pub struct AdminUser(pub User);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(u) = AuthUser::from_request_parts(parts, state).await?;
        if !u.is_admin() {
            return Err(ApiError::forbidden("관리자만 사용할 수 있습니다."));
        }
        Ok(AdminUser(u))
    }
}

// ---------- client IP ----------

/// The end user's IP, resolved once per request by [`client_ip_layer`].
#[derive(Clone, Copy, Debug)]
pub struct ClientIp(pub IpAddr);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ClientIp>()
            .copied()
            .ok_or_else(|| ApiError::internal("client ip missing"))
    }
}

fn header_ip(headers: &HeaderMap, name: &str) -> Option<IpAddr> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

/// Trusts forwarded addresses only from the SvelteKit server (shared token)
/// or, when enabled, from Cloudflare.
pub fn resolve_ip(state: &AppState, headers: &HeaderMap, peer: IpAddr) -> IpAddr {
    if let Some(token) = &state.cfg.internal_token {
        let sent = headers.get("x-acc-internal").and_then(|v| v.to_str().ok());
        if sent == Some(token.as_str()) {
            if let Some(ip) = header_ip(headers, "x-acc-client-ip") {
                return ip;
            }
        }
    }
    if state.cfg.trust_cloudflare {
        if let Some(ip) = header_ip(headers, "cf-connecting-ip") {
            return ip;
        }
    }
    peer
}

pub async fn client_ip_layer(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));
    let ip = resolve_ip(&state, req.headers(), peer);
    req.extensions_mut().insert(ClientIp(ip));
    next.run(req).await
}

/// Rate-limit key for tower-governor, read from [`ClientIp`].
#[derive(Clone)]
pub struct ClientIpKey;

impl tower_governor::key_extractor::KeyExtractor for ClientIpKey {
    type Key = IpAddr;
    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        req.extensions()
            .get::<ClientIp>()
            .map(|c| c.0)
            .ok_or(tower_governor::GovernorError::UnableToExtractKey)
    }
}
