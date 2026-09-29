//! Submitting code and the judge status boards.

use acc_core::queue::Job;
use acc_core::{db as derived, limits, Language, Status};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::problems::{can_see, load_problem};
use crate::auth::{ActiveUser, AuthUser, User, Viewer};
use crate::error::{ApiError, ApiResult};
use crate::page::{self, Page};
use crate::ratelimit;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(submit))
        .routes(routes!(list_submissions))
        .routes(routes!(get_submission))
}

/// Pushes a job for the worker.
pub async fn enqueue(s: &AppState, job: Job) -> ApiResult<()> {
    let mut r = s.redis.clone();
    let _: i64 = r.lpush(acc_core::queue::QUEUE, job.encode()).await?;
    Ok(())
}

pub async fn queue_len(s: &AppState) -> ApiResult<usize> {
    let mut r = s.redis.clone();
    Ok(r.llen(acc_core::queue::QUEUE).await?)
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitBody {
    pub language: String,
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct SubmitResult {
    pub id: i64,
}

/// One submission per user per 10 seconds; refused while the queue is full.
#[utoipa::path(post, path = "/problems/{id}/submit", tag = "submissions", params(("id" = i32, Path)),
    request_body = SubmitBody,
    responses((status = 200, body = SubmitResult), (status = 429, body = crate::error::ErrorBody), (status = 503, body = crate::error::ErrorBody)))]
async fn submit(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
    Json(b): Json<SubmitBody>,
) -> ApiResult<Json<SubmitResult>> {
    let p = load_problem(&s, id)
        .await?
        .filter(|p| can_see(p, Some(&u)))
        .ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    let lang: Language = b
        .language
        .parse()
        .map_err(|_| ApiError::bad_request("지원하지 않는 언어입니다."))?;
    if b.code.trim().is_empty() {
        return Err(ApiError::bad_request("코드가 비어 있습니다."));
    }
    if b.code.len() > limits::MAX_CODE_BYTES {
        return Err(ApiError::bad_request("코드는 64KB 이하여야 합니다."));
    }
    let has_data: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM testcases WHERE problem_id = $1)")
            .bind(p.id)
            .fetch_one(&s.db)
            .await?;
    if !has_data {
        return Err(ApiError::conflict(
            "NO_TESTDATA",
            "테스트 데이터가 없는 문제입니다.",
        ));
    }
    if queue_len(&s).await? >= s.cfg.queue_limit {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "QUEUE_FULL",
            "채점 대기열이 가득 찼습니다. 잠시 후 다시 제출하세요.",
        ));
    }
    if !ratelimit::once_per(&s, &format!("submit:{}", u.id), 10).await? {
        return Err(ApiError::too_many("제출은 10초에 한 번 할 수 있습니다."));
    }
    let mut tx = s.db.begin().await?;
    let sid: i64 = sqlx::query_scalar(
        "INSERT INTO submissions (user_id, problem_id, language, code, code_length) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(u.id)
    .bind(p.id)
    .bind(lang.id())
    .bind(&b.code)
    .bind(b.code.len() as i32)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE problems SET submission_count = submission_count + 1 WHERE id = $1")
        .bind(p.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    derived::refresh_user_problem(&s.db, u.id, p.id).await?;
    enqueue(&s, Job::Submission(sid)).await?;
    Ok(Json(SubmitResult { id: sid }))
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct SubmissionRow {
    pub id: i64,
    /// Null for a deleted account ("탈퇴한 사용자").
    pub handle: Option<String>,
    pub problem_id: i32,
    pub problem_title: String,
    pub language: String,
    pub status: String,
    pub time_ms: Option<i32>,
    pub memory_kb: Option<i32>,
    pub code_length: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SubmissionQuery {
    pub problem_id: Option<i32>,
    pub handle: Option<String>,
    /// AC, WA, ...
    pub status: Option<String>,
    pub language: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

const SUBMISSION_COLUMNS: &str =
    "s.id, u.handle, s.problem_id, p.title AS problem_title, s.language, s.status,
    s.time_ms, s.memory_kb, s.code_length, s.created_at";

#[utoipa::path(get, path = "/submissions", tag = "submissions", params(SubmissionQuery),
    responses((status = 200, body = Page<SubmissionRow>)))]
async fn list_submissions(
    State(s): State<AppState>,
    Viewer(v): Viewer,
    Query(q): Query<SubmissionQuery>,
) -> ApiResult<Json<Page<SubmissionRow>>> {
    let limit = q
        .limit
        .unwrap_or(page::DEFAULT_LIMIT)
        .clamp(1, page::MAX_LIMIT);
    let mut b: QueryBuilder<Postgres> = QueryBuilder::new(format!(
        "SELECT {SUBMISSION_COLUMNS} FROM submissions s JOIN users u ON u.id = s.user_id
         JOIN problems p ON p.id = s.problem_id WHERE TRUE"
    ));
    if !v.as_ref().is_some_and(User::is_admin) {
        b.push(" AND p.status = 'PUBLIC'");
    }
    if let Some(pid) = q.problem_id {
        b.push(" AND s.problem_id = ").push_bind(pid);
    }
    if let Some(h) = &q.handle {
        b.push(" AND u.handle = ").push_bind(h.to_lowercase());
    }
    if let Some(st) = &q.status {
        let st: Status = st
            .parse()
            .map_err(|_| ApiError::bad_request("알 수 없는 상태입니다."))?;
        b.push(" AND s.status = ").push_bind(st.as_str());
    }
    if let Some(l) = &q.language {
        let l: Language = l
            .parse()
            .map_err(|_| ApiError::bad_request("지원하지 않는 언어입니다."))?;
        b.push(" AND s.language = ").push_bind(l.id());
    }
    if let Some(c) = &q.cursor {
        let (_, id) = page::decode(c)?;
        b.push(" AND s.id < ").push_bind(id);
    }
    b.push(" ORDER BY s.id DESC LIMIT ").push_bind(limit + 1);
    let rows: Vec<SubmissionRow> = b.build_query_as().fetch_all(&s.db).await?;
    Ok(Json(page::finish(rows, limit, |r| (0, r.id))))
}

#[derive(Serialize, ToSchema)]
pub struct SubmissionDetail {
    #[serde(flatten)]
    pub row: SubmissionRow,
    pub judged_at: Option<DateTime<Utc>>,
    /// Present when the viewer may read it: the author, admins, and users who solved the problem.
    pub code: Option<String>,
    /// Compile output, for the author of a CE submission.
    pub compile_message: Option<String>,
    pub is_mine: bool,
}

#[utoipa::path(get, path = "/submissions/{id}", tag = "submissions", params(("id" = i64, Path)),
    responses((status = 200, body = SubmissionDetail), (status = 404, body = crate::error::ErrorBody)))]
async fn get_submission(
    State(s): State<AppState>,
    AuthUser(u): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<SubmissionDetail>> {
    #[derive(FromRow)]
    struct Extra {
        user_id: i64,
        code: String,
        message: Option<String>,
        judged_at: Option<DateTime<Utc>>,
        problem_status: String,
    }
    let row = sqlx::query_as::<_, SubmissionRow>(&format!(
        "SELECT {SUBMISSION_COLUMNS} FROM submissions s JOIN users u ON u.id = s.user_id
         JOIN problems p ON p.id = s.problem_id WHERE s.id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::not_found("제출을 찾을 수 없습니다."))?;
    let extra = sqlx::query_as::<_, Extra>(
        "SELECT s.user_id, s.code, s.message, s.judged_at, p.status AS problem_status
         FROM submissions s JOIN problems p ON p.id = s.problem_id WHERE s.id = $1",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    let is_mine = extra.user_id == u.id;
    if extra.problem_status != "PUBLIC" && !is_mine && !u.is_admin() {
        return Err(ApiError::not_found("제출을 찾을 수 없습니다."));
    }
    let solved: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM user_problem WHERE user_id = $1 AND problem_id = $2 AND state = 'SOLVED')",
    )
    .bind(u.id)
    .bind(row.problem_id)
    .fetch_one(&s.db)
    .await?;
    let can_code = is_mine || u.is_admin() || solved;
    let compile_message = if (is_mine || u.is_admin()) && row.status == "CE" {
        extra.message.clone()
    } else {
        None
    };
    Ok(Json(SubmissionDetail {
        row,
        judged_at: extra.judged_at,
        code: can_code.then_some(extra.code),
        compile_message,
        is_mine,
    }))
}
