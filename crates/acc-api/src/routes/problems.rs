//! Public problem list and problem pages.

use acc_core::level;
use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::auth::{User, Viewer};
use crate::error::{ApiError, ApiResult};
use crate::page::{self, Page};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_problems))
        .routes(routes!(get_problem))
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct Sample {
    pub input: String,
    pub output: String,
}

/// Statement sections, Markdown with KaTeX math.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct Statement {
    pub legend: String,
    pub input: String,
    pub output: String,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub samples: Vec<Sample>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct ProblemSummary {
    pub id: i32,
    pub title: String,
    pub level: i16,
    pub source: String,
    pub solved_count: i32,
    pub submission_count: i32,
    /// Viewer's state: SOLVED, TRIED or null.
    pub my_state: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ProblemListQuery {
    /// id (default), level, solved
    pub sort: Option<String>,
    /// asc (default for id) or desc
    pub order: Option<String>,
    /// Group name: unrated, natural, integer, rational, real, complex, quaternion
    pub group: Option<String>,
    /// Title search, or an exact problem number.
    pub q: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

fn group_range(name: &str) -> Option<(i16, i16)> {
    if name.eq_ignore_ascii_case("unrated") {
        return Some((0, 0));
    }
    level::GROUPS
        .iter()
        .position(|g| g.name.eq_ignore_ascii_case(name))
        .map(|i| (i as i16 * 5 + 1, i as i16 * 5 + 5))
}

#[utoipa::path(get, path = "/problems", tag = "problems", params(ProblemListQuery),
    responses((status = 200, body = Page<ProblemSummary>)))]
async fn list_problems(
    State(s): State<AppState>,
    Viewer(v): Viewer,
    Query(q): Query<ProblemListQuery>,
) -> ApiResult<Json<Page<ProblemSummary>>> {
    let limit = q
        .limit
        .unwrap_or(page::DEFAULT_LIMIT)
        .clamp(1, page::MAX_LIMIT);
    let key = match q.sort.as_deref().unwrap_or("id") {
        "id" => "p.id",
        "level" => "p.level",
        "solved" => "p.solved_count",
        _ => {
            return Err(ApiError::bad_request(
                "sort는 id, level, solved 중 하나입니다.",
            ))
        }
    };
    let desc = match q.order.as_deref() {
        Some("desc") => true,
        Some("asc") | None => false,
        _ => return Err(ApiError::bad_request("order는 asc 또는 desc입니다.")),
    };

    let mut b: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT p.id, p.title, p.level, p.source, p.solved_count, p.submission_count, up.state AS my_state
         FROM problems p LEFT JOIN user_problem up ON up.problem_id = p.id AND up.user_id = ",
    );
    b.push_bind(v.as_ref().map(|u| u.id).unwrap_or(0));
    b.push(" WHERE p.status = 'PUBLIC'");
    if let Some(g) = &q.group {
        let (lo, hi) =
            group_range(g).ok_or_else(|| ApiError::bad_request("알 수 없는 그룹입니다."))?;
        b.push(" AND p.level BETWEEN ")
            .push_bind(lo)
            .push(" AND ")
            .push_bind(hi);
    }
    if let Some(term) = q.q.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        match term.parse::<i32>() {
            Ok(id) => {
                b.push(" AND p.id = ").push_bind(id);
            }
            Err(_) => {
                let like = format!(
                    "%{}%",
                    term.replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_")
                );
                b.push(" AND p.title ILIKE ").push_bind(like);
            }
        }
    }
    if let Some(c) = &q.cursor {
        let (ck, cid) = page::decode(c)?;
        b.push(format!(
            " AND ({key}::bigint, p.id::bigint) {} (",
            if desc { "<" } else { ">" }
        ));
        b.push_bind(ck).push(", ").push_bind(cid).push(")");
    }
    let dir = if desc { "DESC" } else { "ASC" };
    b.push(format!(" ORDER BY {key} {dir}, p.id {dir} LIMIT "));
    b.push_bind(limit + 1);

    let rows: Vec<ProblemSummary> = b.build_query_as().fetch_all(&s.db).await?;
    let sort = q.sort.clone().unwrap_or_else(|| "id".into());
    Ok(Json(page::finish(rows, limit, |r| {
        let k = match sort.as_str() {
            "level" => i64::from(r.level),
            "solved" => i64::from(r.solved_count),
            _ => i64::from(r.id),
        };
        (k, i64::from(r.id))
    })))
}

#[derive(Serialize, ToSchema)]
pub struct ProblemDetail {
    pub id: i32,
    pub title: String,
    /// DRAFT, REVIEW or PUBLIC. Non-public problems are visible to the author and admins only.
    pub status: String,
    pub source: String,
    pub level: i16,
    pub time_limit_ms: i32,
    pub memory_limit_kb: i32,
    pub statement: Statement,
    pub author: Option<String>,
    pub solved_count: i32,
    pub submission_count: i32,
    /// Share of judged submissions that were AC, 0..=1.
    pub accept_rate: f64,
    pub my_state: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
pub(crate) struct ProblemRow {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub source: String,
    pub level: i16,
    pub time_limit_ms: i32,
    pub memory_limit_kb: i32,
    pub statement: sqlx::types::Json<Statement>,
    pub author_id: i64,
    pub author: Option<String>,
    pub solved_count: i32,
    pub submission_count: i32,
    pub published_at: Option<DateTime<Utc>>,
}

pub(crate) async fn load_problem(s: &AppState, id: i32) -> ApiResult<Option<ProblemRow>> {
    Ok(sqlx::query_as::<_, ProblemRow>(
        "SELECT p.id, p.title, p.status, p.source, p.level, p.time_limit_ms, p.memory_limit_kb, p.statement,
                p.author_id, u.handle AS author, p.solved_count, p.submission_count, p.published_at
         FROM problems p JOIN users u ON u.id = p.author_id WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?)
}

/// Public problems for everyone; others for their author and admins.
pub(crate) fn can_see(p: &ProblemRow, v: Option<&User>) -> bool {
    p.status == "PUBLIC" || v.is_some_and(|u| u.is_admin() || u.id == p.author_id)
}

#[utoipa::path(get, path = "/problems/{id}", tag = "problems", params(("id" = i32, Path)),
    responses((status = 200, body = ProblemDetail), (status = 404, body = crate::error::ErrorBody)))]
async fn get_problem(
    State(s): State<AppState>,
    Viewer(v): Viewer,
    Path(id): Path<i32>,
) -> ApiResult<Json<ProblemDetail>> {
    let p = load_problem(&s, id)
        .await?
        .filter(|p| can_see(p, v.as_ref()))
        .ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    let (judged, ac): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE status NOT IN ('PENDING', 'JUDGING', 'SE')),
                count(*) FILTER (WHERE status = 'AC')
         FROM submissions WHERE problem_id = $1",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    let my_state: Option<String> = match &v {
        Some(u) => {
            sqlx::query_scalar(
                "SELECT state FROM user_problem WHERE user_id = $1 AND problem_id = $2",
            )
            .bind(u.id)
            .bind(id)
            .fetch_optional(&s.db)
            .await?
        }
        None => None,
    };
    Ok(Json(ProblemDetail {
        id: p.id,
        title: p.title,
        status: p.status,
        source: p.source,
        level: p.level,
        time_limit_ms: p.time_limit_ms,
        memory_limit_kb: p.memory_limit_kb,
        statement: p.statement.0,
        author: p.author,
        solved_count: p.solved_count,
        submission_count: p.submission_count,
        accept_rate: if judged > 0 {
            ac as f64 / judged as f64
        } else {
            0.0
        },
        my_state,
        published_at: p.published_at,
    }))
}
