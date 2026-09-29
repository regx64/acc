use acc_core::{level, Language};
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::AdminUser;
use crate::error::ApiResult;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(site_info))
        .routes(routes!(set_banner))
}

#[utoipa::path(get, path = "/health", tag = "site", responses((status = 200, body = String)))]
async fn health(State(s): State<AppState>) -> ApiResult<&'static str> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    Ok("ok")
}

#[derive(Serialize, ToSchema)]
pub struct LanguageInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct GroupInfo {
    pub symbol: String,
    pub name: String,
    pub color: String,
    pub min_level: u8,
    pub max_level: u8,
}

#[derive(Serialize, ToSchema)]
pub struct SiteInfo {
    /// Maintenance notice shown at the top of every page.
    pub banner: Option<String>,
    pub languages: Vec<LanguageInfo>,
    pub groups: Vec<GroupInfo>,
    pub tier_coefficient: i64,
}

#[utoipa::path(get, path = "/site", tag = "site", responses((status = 200, body = SiteInfo)))]
async fn site_info(State(s): State<AppState>) -> ApiResult<Json<SiteInfo>> {
    let banner: Option<String> =
        sqlx::query_scalar("SELECT value FROM site_settings WHERE key = 'banner'")
            .fetch_optional(&s.db)
            .await?
            .filter(|v: &String| !v.trim().is_empty());
    Ok(Json(SiteInfo {
        banner,
        languages: Language::ALL
            .iter()
            .map(|l| LanguageInfo {
                id: l.id().into(),
                name: l.display_name().into(),
            })
            .collect(),
        groups: level::GROUPS
            .iter()
            .enumerate()
            .map(|(i, g)| GroupInfo {
                symbol: g.symbol.into(),
                name: g.name.into(),
                color: g.color.into(),
                min_level: i as u8 * 5 + 1,
                max_level: i as u8 * 5 + 5,
            })
            .collect(),
        tier_coefficient: level::TIER_COEFF,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct BannerBody {
    /// Empty to clear.
    pub text: String,
}

#[utoipa::path(put, path = "/admin/banner", tag = "admin", request_body = BannerBody, responses((status = 204)))]
async fn set_banner(
    State(s): State<AppState>,
    _: AdminUser,
    Json(b): Json<BannerBody>,
) -> ApiResult<axum::http::StatusCode> {
    sqlx::query("INSERT INTO site_settings (key, value) VALUES ('banner', $1) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value")
        .bind(b.text.trim())
        .execute(&s.db)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
