//! Redis-backed rules from the plan (exact per-user / per-IP windows).

use redis::AsyncCommands;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Allows one action per `window_secs` for `key`. Returns false when blocked.
pub async fn once_per(state: &AppState, key: &str, window_secs: u64) -> ApiResult<bool> {
    let mut r = state.redis.clone();
    let set: Option<String> = redis::cmd("SET")
        .arg(format!("acc:rl:{key}"))
        .arg(1)
        .arg("NX")
        .arg("EX")
        .arg(window_secs)
        .query_async(&mut r)
        .await?;
    Ok(set.is_some())
}

pub const LOGIN_FAILURES_PER_MINUTE: i64 = 5;

fn login_key(ip: &std::net::IpAddr) -> String {
    format!("acc:rl:login:{ip}")
}

/// Refuses when this IP already failed 5 logins in the last minute.
pub async fn check_login(state: &AppState, ip: &std::net::IpAddr) -> ApiResult<()> {
    let mut r = state.redis.clone();
    let n: Option<i64> = r.get(login_key(ip)).await?;
    if n.unwrap_or(0) >= LOGIN_FAILURES_PER_MINUTE {
        return Err(ApiError::too_many(
            "로그인 시도가 너무 많습니다. 1분 후 다시 시도하세요.",
        ));
    }
    Ok(())
}

pub async fn record_login_failure(state: &AppState, ip: &std::net::IpAddr) -> ApiResult<()> {
    let mut r = state.redis.clone();
    let key = login_key(ip);
    let n: i64 = r.incr(&key, 1).await?;
    if n == 1 {
        let _: bool = r.expire(&key, 60).await?;
    }
    Ok(())
}
