//! Public profiles and the rating ranking.

use acc_core::level;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::error::{ApiError, ApiResult};
use crate::page::{self, Page, PageQuery};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(profile))
        .routes(routes!(ranking))
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct ProblemRef {
    pub id: i32,
    pub title: String,
    pub level: i16,
}

#[derive(Serialize, ToSchema)]
pub struct Profile {
    pub handle: String,
    pub rating: i32,
    pub tier: i16,
    /// 1-based position in the ranking.
    pub rank: i64,
    pub joined_at: DateTime<Utc>,
    pub solved: Vec<ProblemRef>,
    /// Tried but not solved.
    pub failed: Vec<ProblemRef>,
    /// Solved count per level, index 0..=30.
    pub level_counts: Vec<i64>,
    pub submission_count: i64,
    /// Rating needed for the next tier; null at the top.
    pub next_tier_rating: Option<i64>,
}

#[utoipa::path(get, path = "/users/{handle}", tag = "users",
    params(("handle" = String, Path)),
    responses((status = 200, body = Profile), (status = 404, body = crate::error::ErrorBody)))]
async fn profile(
    State(s): State<AppState>,
    Path(handle): Path<String>,
) -> ApiResult<Json<Profile>> {
    let handle = handle.to_lowercase();
    let row: Option<(i64, i32, i16, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, rating, tier, created_at FROM users WHERE handle = $1 AND NOT deleted",
    )
    .bind(&handle)
    .fetch_optional(&s.db)
    .await?;
    let Some((uid, rating, tier, joined_at)) = row else {
        let moved: Option<String> = sqlx::query_scalar(
            "SELECT u.handle FROM reserved_handles r JOIN users u ON u.id = r.user_id
             WHERE r.handle = $1 AND r.until > now() AND NOT u.deleted",
        )
        .bind(&handle)
        .fetch_optional(&s.db)
        .await?
        .flatten();
        return Err(match moved {
            Some(to) => ApiError {
                status: StatusCode::NOT_FOUND,
                code: "HANDLE_MOVED",
                message: format!("핸들이 {to}(으)로 바뀌었습니다."),
                location: Some(to),
            },
            None => ApiError::not_found("사용자를 찾을 수 없습니다."),
        });
    };
    let problems = |state: &'static str| {
        sqlx::query_as::<_, ProblemRef>(
            "SELECT p.id, p.title, p.level FROM user_problem up JOIN problems p ON p.id = up.problem_id
             WHERE up.user_id = $1 AND up.state = $2 AND p.status = 'PUBLIC' ORDER BY p.id",
        )
        .bind(uid)
        .bind(state)
        .fetch_all(&s.db)
    };
    let solved = problems("SOLVED").await?;
    let failed = problems("TRIED").await?;
    let rank: i64 =
        sqlx::query_scalar("SELECT count(*) + 1 FROM users WHERE NOT deleted AND rating > $1")
            .bind(rating)
            .fetch_one(&s.db)
            .await?;
    let submission_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM submissions WHERE user_id = $1")
            .bind(uid)
            .fetch_one(&s.db)
            .await?;
    let mut level_counts = vec![0i64; 31];
    for p in &solved {
        level_counts[p.level.clamp(0, 30) as usize] += 1;
    }
    let tier_u8 = tier.clamp(0, 30) as u8;
    let next_tier_rating = (tier_u8 < level::MAX_LEVEL).then(|| level::tier_threshold(tier_u8 + 1));
    Ok(Json(Profile {
        handle,
        rating,
        tier,
        rank,
        joined_at,
        solved,
        failed,
        level_counts,
        submission_count,
        next_tier_rating,
    }))
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct RankingRow {
    pub rank: i64,
    pub id: i64,
    pub handle: String,
    pub rating: i32,
    pub tier: i16,
    pub solved_count: i64,
}

#[utoipa::path(get, path = "/ranking", tag = "users", params(PageQuery), responses((status = 200, body = Page<RankingRow>)))]
async fn ranking(
    State(s): State<AppState>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<Page<RankingRow>>> {
    let limit = q.limit();
    let (after_rating, after_id) = match &q.cursor {
        Some(c) => page::decode(c)?,
        None => (i64::MAX, 0),
    };
    let rows = sqlx::query_as::<_, RankingRow>(
        "SELECT * FROM (
            SELECT rank() OVER (ORDER BY u.rating DESC) AS rank, u.id, u.handle, u.rating, u.tier,
                   (SELECT count(*) FROM user_problem up JOIN problems p ON p.id = up.problem_id
                     WHERE up.user_id = u.id AND up.state = 'SOLVED' AND p.status = 'PUBLIC') AS solved_count
            FROM users u WHERE NOT u.deleted
         ) r
         WHERE r.rating::bigint < $1 OR (r.rating::bigint = $1 AND r.id > $2)
         ORDER BY r.rating DESC, r.id
         LIMIT $3",
    )
    .bind(after_rating)
    .bind(after_id)
    .bind(limit + 1)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(page::finish(rows, limit, |r| {
        (i64::from(r.rating), r.id)
    })))
}
