//! `/admin/*`: review, problem level/status, rejudge, suspensions, reports.

use acc_core::db as derived;
use acc_core::queue::Job;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::submissions::{enqueue, queue_len};
use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(overview))
        .routes(routes!(admin_list_problems))
        .routes(routes!(admin_update_problem))
        .routes(routes!(decide))
        .routes(routes!(rejudge_problem))
        .routes(routes!(rejudge_submission))
        .routes(routes!(list_users))
        .routes(routes!(suspend))
        .routes(routes!(unsuspend))
        .routes(routes!(list_reports))
        .routes(routes!(resolve_report))
        .routes(routes!(post_visibility))
        .routes(routes!(comment_visibility))
        .routes(routes!(list_typos))
        .routes(routes!(resolve_typo))
}

#[derive(Serialize, ToSchema)]
pub struct Overview {
    pub queue_length: usize,
    pub queue_limit: usize,
    pub judging: i64,
    /// Judged submissions in the last 24 hours.
    pub judged_24h: i64,
    /// SE share of the last 24 hours' judged submissions, 0..=1.
    pub system_error_rate_24h: f64,
    pub pending_reviews: i64,
    pub open_reports: i64,
    pub open_typos: i64,
    pub users: i64,
    pub public_problems: i64,
}

#[utoipa::path(get, path = "/admin/overview", tag = "admin", responses((status = 200, body = Overview)))]
async fn overview(State(s): State<AppState>, _: AdminUser) -> ApiResult<Json<Overview>> {
    let (judging, judged, se): (i64, i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE status IN ('PENDING', 'JUDGING')),
                count(*) FILTER (WHERE judged_at > now() - interval '24 hours'),
                count(*) FILTER (WHERE judged_at > now() - interval '24 hours' AND status = 'SE')
         FROM submissions WHERE status IN ('PENDING', 'JUDGING') OR judged_at > now() - interval '24 hours'",
    )
    .fetch_one(&s.db)
    .await?;
    let (pending_reviews, open_reports, open_typos, users, public_problems): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM problems WHERE status = 'REVIEW'),
                (SELECT count(DISTINCT (target_type, target_id)) FROM reports WHERE NOT resolved),
                (SELECT count(*) FROM posts WHERE kind = 'TYPO' AND NOT resolved AND NOT hidden),
                (SELECT count(*) FROM users WHERE NOT deleted),
                (SELECT count(*) FROM problems WHERE status = 'PUBLIC')",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(Json(Overview {
        queue_length: queue_len(&s).await?,
        queue_limit: s.cfg.queue_limit,
        judging,
        judged_24h: judged,
        system_error_rate_24h: if judged > 0 {
            se as f64 / judged as f64
        } else {
            0.0
        },
        pending_reviews,
        open_reports,
        open_typos,
        users,
        public_problems,
    }))
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct AdminProblemRow {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub source: String,
    pub level: i16,
    pub proposed_level: Option<i16>,
    pub author: Option<String>,
    pub validation_status: Option<String>,
    pub solved_count: i32,
    pub testcase_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminProblemQuery {
    /// DRAFT, REVIEW or PUBLIC
    pub status: Option<String>,
}

#[utoipa::path(get, path = "/admin/problems", tag = "admin", params(AdminProblemQuery), responses((status = 200, body = Vec<AdminProblemRow>)))]
async fn admin_list_problems(
    State(s): State<AppState>,
    _: AdminUser,
    Query(q): Query<AdminProblemQuery>,
) -> ApiResult<Json<Vec<AdminProblemRow>>> {
    let rows = sqlx::query_as::<_, AdminProblemRow>(
        "SELECT p.id, p.title, p.status, p.source, p.level, p.proposed_level, u.handle AS author,
                p.validation_status, p.solved_count,
                (SELECT count(*) FROM testcases t WHERE t.problem_id = p.id) AS testcase_count, p.created_at
         FROM problems p JOIN users u ON u.id = p.author_id
         WHERE $1::text IS NULL OR p.status = $1
         ORDER BY (p.status = 'REVIEW') DESC, p.id DESC LIMIT 500",
    )
    .bind(q.status)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
pub struct AdminProblemPatch {
    pub level: Option<i16>,
    /// DRAFT, REVIEW or PUBLIC
    pub status: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Recomputed {
    /// Users whose rating was recomputed.
    pub users: usize,
}

/// Changing the level or visibility recomputes every solver's rating.
#[utoipa::path(patch, path = "/admin/problems/{id}", tag = "admin", params(("id" = i32, Path)), request_body = AdminProblemPatch,
    responses((status = 200, body = Recomputed)))]
async fn admin_update_problem(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i32>,
    Json(b): Json<AdminProblemPatch>,
) -> ApiResult<Json<Recomputed>> {
    if b.level.is_some_and(|l| !(0..=30).contains(&l)) {
        return Err(ApiError::bad_request("레벨은 0~30입니다."));
    }
    if b.status
        .as_deref()
        .is_some_and(|st| !["DRAFT", "REVIEW", "PUBLIC"].contains(&st))
    {
        return Err(ApiError::bad_request(
            "상태는 DRAFT, REVIEW, PUBLIC 중 하나입니다.",
        ));
    }
    let n = sqlx::query(
        "UPDATE problems SET level = COALESCE($2, level), status = COALESCE($3, status),
                published_at = CASE WHEN $3 = 'PUBLIC' AND published_at IS NULL THEN now() ELSE published_at END
         WHERE id = $1",
    )
    .bind(id)
    .bind(b.level)
    .bind(&b.status)
    .execute(&s.db)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(ApiError::not_found("문제를 찾을 수 없습니다."));
    }
    let users = derived::recompute_solvers(&s.db, id).await?;
    Ok(Json(Recomputed { users }))
}

#[derive(Deserialize, ToSchema)]
pub struct Decision {
    pub approve: bool,
    /// Final level; required when approving.
    pub level: Option<i16>,
    pub comment: Option<String>,
}

/// Approve (publish with a level) or reject (back to draft with a comment).
/// Also used to publish official problems directly from draft.
#[utoipa::path(post, path = "/admin/problems/{id}/decision", tag = "admin", params(("id" = i32, Path)), request_body = Decision,
    responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn decide(
    State(s): State<AppState>,
    AdminUser(a): AdminUser,
    Path(id): Path<i32>,
    Json(b): Json<Decision>,
) -> ApiResult<StatusCode> {
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT status, validation_status FROM problems WHERE id = $1")
            .bind(id)
            .fetch_optional(&s.db)
            .await?;
    let (status, validation) =
        row.ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    if status == "PUBLIC" {
        return Err(ApiError::conflict(
            "ALREADY_PUBLIC",
            "이미 공개된 문제입니다.",
        ));
    }
    let comment = b
        .comment
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());
    let mut tx = s.db.begin().await?;
    if b.approve {
        let level = b
            .level
            .ok_or_else(|| ApiError::bad_request("승인할 때는 레벨을 정해야 합니다."))?;
        if !(0..=30).contains(&level) {
            return Err(ApiError::bad_request("레벨은 0~30입니다."));
        }
        if validation.as_deref() != Some("PASSED") {
            return Err(ApiError::conflict(
                "NOT_VALIDATED",
                "정해 검증을 통과한 문제만 공개할 수 있습니다.",
            ));
        }
        sqlx::query(
            "UPDATE problems SET status = 'PUBLIC', level = $2, published_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(level)
        .execute(&mut *tx)
        .await?;
    } else {
        if comment.is_none() {
            return Err(ApiError::bad_request("반려 사유를 적어 주세요."));
        }
        sqlx::query("UPDATE problems SET status = 'DRAFT' WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO problem_reviews (problem_id, reviewer_id, decision, comment) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(a.id)
        .bind(if b.approve { "APPROVED" } else { "REJECTED" })
        .bind(comment)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    if b.approve {
        // Authors' and testers' earlier AC now count.
        derived::recompute_solvers(&s.db, id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema)]
pub struct Requeued {
    pub submissions: usize,
}

async fn requeue(s: &AppState, ids: Vec<i64>) -> ApiResult<usize> {
    for &id in &ids {
        enqueue(s, Job::Submission(id)).await?;
    }
    Ok(ids.len())
}

/// Rejudges every submission of a problem. Ratings follow the new verdicts.
#[utoipa::path(post, path = "/admin/problems/{id}/rejudge", tag = "admin", params(("id" = i32, Path)), responses((status = 200, body = Requeued)))]
async fn rejudge_problem(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i32>,
) -> ApiResult<Json<Requeued>> {
    let ids: Vec<i64> = sqlx::query_scalar(
        "UPDATE submissions SET status = 'PENDING', time_ms = NULL, memory_kb = NULL, message = NULL, judged_at = NULL
         WHERE problem_id = $1 RETURNING id",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    let mut ids = ids;
    ids.sort_unstable();
    Ok(Json(Requeued {
        submissions: requeue(&s, ids).await?,
    }))
}

#[utoipa::path(post, path = "/admin/submissions/{id}/rejudge", tag = "admin", params(("id" = i64, Path)), responses((status = 200, body = Requeued)))]
async fn rejudge_submission(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Requeued>> {
    let ids: Vec<i64> = sqlx::query_scalar(
        "UPDATE submissions SET status = 'PENDING', time_ms = NULL, memory_kb = NULL, message = NULL, judged_at = NULL
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    if ids.is_empty() {
        return Err(ApiError::not_found("제출을 찾을 수 없습니다."));
    }
    Ok(Json(Requeued {
        submissions: requeue(&s, ids).await?,
    }))
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub handle: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub suspended: bool,
    pub suspend_reason: Option<String>,
    pub rating: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UserSearch {
    /// Part of a handle or email.
    pub q: Option<String>,
}

#[utoipa::path(get, path = "/admin/users", tag = "admin", params(UserSearch), responses((status = 200, body = Vec<AdminUserRow>)))]
async fn list_users(
    State(s): State<AppState>,
    _: AdminUser,
    Query(q): Query<UserSearch>,
) -> ApiResult<Json<Vec<AdminUserRow>>> {
    let like = format!(
        "%{}%",
        q.q.unwrap_or_default()
            .trim()
            .to_lowercase()
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let rows = sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, handle, email, role, email_verified, suspended, suspend_reason, rating, created_at
         FROM users WHERE NOT deleted AND (handle LIKE $1 OR email LIKE $1)
         ORDER BY suspended DESC, id DESC LIMIT 100",
    )
    .bind(like)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
pub struct SuspendBody {
    pub reason: String,
}

/// Suspended users can still sign in but cannot submit or write.
#[utoipa::path(post, path = "/admin/users/{id}/suspend", tag = "admin", params(("id" = i64, Path)), request_body = SuspendBody, responses((status = 204)))]
async fn suspend(
    State(s): State<AppState>,
    AdminUser(a): AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<SuspendBody>,
) -> ApiResult<StatusCode> {
    if id == a.id {
        return Err(ApiError::bad_request("자기 자신은 정지할 수 없습니다."));
    }
    let reason = b.reason.trim();
    if reason.is_empty() {
        return Err(ApiError::bad_request("정지 사유를 적어 주세요."));
    }
    sqlx::query(
        "UPDATE users SET suspended = TRUE, suspend_reason = $2 WHERE id = $1 AND NOT deleted",
    )
    .bind(id)
    .bind(reason)
    .execute(&s.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/admin/users/{id}/unsuspend", tag = "admin", params(("id" = i64, Path)), responses((status = 204)))]
async fn unsuspend(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    sqlx::query("UPDATE users SET suspended = FALSE, suspend_reason = NULL WHERE id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct ReportRow {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    /// Post the target belongs to, for linking.
    pub post_id: Option<i64>,
    pub excerpt: Option<String>,
    pub target_hidden: Option<bool>,
    pub reason: String,
    pub reporter: Option<String>,
    pub report_count: i64,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(get, path = "/admin/reports", tag = "admin", responses((status = 200, body = Vec<ReportRow>)))]
async fn list_reports(State(s): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<ReportRow>>> {
    let rows = sqlx::query_as::<_, ReportRow>(
        "SELECT r.id, r.target_type, r.target_id,
                CASE WHEN r.target_type = 'POST' THEN r.target_id ELSE c.post_id END AS post_id,
                left(COALESCE(p.title || ' — ' || p.body, c.body), 200) AS excerpt,
                COALESCE(p.hidden, c.hidden) AS target_hidden,
                r.reason, u.handle AS reporter,
                count(*) OVER (PARTITION BY r.target_type, r.target_id) AS report_count,
                r.created_at
         FROM reports r
         JOIN users u ON u.id = r.reporter_id
         LEFT JOIN posts p ON r.target_type = 'POST' AND p.id = r.target_id
         LEFT JOIN comments c ON r.target_type = 'COMMENT' AND c.id = r.target_id
         WHERE NOT r.resolved ORDER BY r.id DESC LIMIT 300",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
pub struct ResolveBody {
    /// true hides the target, false restores it.
    pub hide: bool,
}

/// Closes every open report on the same target.
#[utoipa::path(post, path = "/admin/reports/{id}/resolve", tag = "admin", params(("id" = i64, Path)), request_body = ResolveBody, responses((status = 204)))]
async fn resolve_report(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<ResolveBody>,
) -> ApiResult<StatusCode> {
    let target: Option<(String, i64)> =
        sqlx::query_as("SELECT target_type, target_id FROM reports WHERE id = $1")
            .bind(id)
            .fetch_optional(&s.db)
            .await?;
    let (tt, tid) = target.ok_or_else(|| ApiError::not_found("신고를 찾을 수 없습니다."))?;
    let table = if tt == "POST" { "posts" } else { "comments" };
    let mut tx = s.db.begin().await?;
    sqlx::query(&format!("UPDATE {table} SET hidden = $2 WHERE id = $1"))
        .bind(tid)
        .bind(b.hide)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE reports SET resolved = TRUE WHERE target_type = $1 AND target_id = $2")
        .bind(&tt)
        .bind(tid)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct VisibilityBody {
    pub hidden: bool,
}

#[utoipa::path(post, path = "/admin/posts/{id}/visibility", tag = "admin", params(("id" = i64, Path)), request_body = VisibilityBody, responses((status = 204)))]
async fn post_visibility(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<VisibilityBody>,
) -> ApiResult<StatusCode> {
    sqlx::query("UPDATE posts SET hidden = $2 WHERE id = $1")
        .bind(id)
        .bind(b.hidden)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/admin/comments/{id}/visibility", tag = "admin", params(("id" = i64, Path)), request_body = VisibilityBody, responses((status = 204)))]
async fn comment_visibility(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<VisibilityBody>,
) -> ApiResult<StatusCode> {
    sqlx::query("UPDATE comments SET hidden = $2 WHERE id = $1")
        .bind(id)
        .bind(b.hidden)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct TypoRow {
    pub id: i64,
    pub problem_id: i32,
    pub title: String,
    pub handle: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(get, path = "/admin/typos", tag = "admin", responses((status = 200, body = Vec<TypoRow>)))]
async fn list_typos(State(s): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<TypoRow>>> {
    let rows = sqlx::query_as::<_, TypoRow>(
        "SELECT p.id, p.problem_id, p.title, u.handle, p.created_at FROM posts p JOIN users u ON u.id = p.user_id
         WHERE p.kind = 'TYPO' AND NOT p.resolved AND NOT p.hidden ORDER BY p.id",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[utoipa::path(post, path = "/admin/posts/{id}/resolve", tag = "admin", params(("id" = i64, Path)), responses((status = 204)))]
async fn resolve_typo(
    State(s): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    sqlx::query("UPDATE posts SET resolved = TRUE WHERE id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
