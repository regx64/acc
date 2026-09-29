//! Cursor pagination. A cursor is an opaque token wrapping the last row's
//! sort key and id.

use base64::Engine;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::error::ApiError;

pub const DEFAULT_LIMIT: i64 = 50;
pub const MAX_LIMIT: i64 = 100;

#[derive(Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Pass as `cursor` to get the next page; absent on the last page.
    pub next_cursor: Option<String>,
}

#[derive(Deserialize, IntoParams, Default)]
#[into_params(parameter_in = Query)]
pub struct PageQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

impl PageQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }
}

pub fn encode(key: i64, id: i64) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{key}:{id}"))
}

pub fn decode(cursor: &str) -> Result<(i64, i64), ApiError> {
    let bad = || ApiError::bad_request("잘못된 커서입니다.");
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| bad())?;
    let s = String::from_utf8(raw).map_err(|_| bad())?;
    let (k, id) = s.split_once(':').ok_or_else(bad)?;
    Ok((
        k.parse().map_err(|_| bad())?,
        id.parse().map_err(|_| bad())?,
    ))
}

/// Trims the extra probe row and builds the page.
pub fn finish<T>(mut rows: Vec<T>, limit: i64, key: impl Fn(&T) -> (i64, i64)) -> Page<T> {
    let more = rows.len() as i64 > limit;
    if more {
        rows.truncate(limit as usize);
    }
    let next_cursor = if more {
        rows.last().map(|r| {
            let (k, id) = key(r);
            encode(k, id)
        })
    } else {
        None
    };
    Page {
        items: rows,
        next_cursor,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn roundtrip() {
        let c = super::encode(-5, 1234);
        assert_eq!(super::decode(&c).unwrap(), (-5, 1234));
        assert!(super::decode("!!").is_err());
    }
}
