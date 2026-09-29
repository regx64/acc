//! Per-problem board: questions, counterexamples and typo reports.

use acc_core::{limits, Language};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::problems::{can_see, load_problem};
use crate::auth::{ActiveUser, User, Viewer};
use crate::error::{ApiError, ApiResult};
use crate::page::{self, Page, PageQuery};
use crate::ratelimit;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_posts, create_post))
        .routes(routes!(get_post, delete_post))
        .routes(routes!(create_comment))
        .routes(routes!(delete_comment))
        .routes(routes!(report))
}

const KINDS: [&str; 3] = ["QUESTION", "COUNTEREXAMPLE", "TYPO"];

#[derive(Serialize, ToSchema, FromRow)]
pub struct PostSummary {
    pub id: i64,
    pub problem_id: i32,
    /// QUESTION, COUNTEREXAMPLE or TYPO
    pub kind: String,
    pub title: String,
    pub handle: Option<String>,
    pub comment_count: i32,
    pub hidden: bool,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(get, path = "/problems/{id}/posts", tag = "board", params(("id" = i32, Path), PageQuery),
    responses((status = 200, body = Page<PostSummary>)))]
async fn list_posts(
    State(s): State<AppState>,
    Viewer(v): Viewer,
    Path(id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<Page<PostSummary>>> {
    let limit = q.limit();
    let before = match &q.cursor {
        Some(c) => page::decode(c)?.1,
        None => i64::MAX,
    };
    let admin = v.as_ref().is_some_and(User::is_admin);
    let rows = sqlx::query_as::<_, PostSummary>(
        "SELECT p.id, p.problem_id, p.kind, p.title, u.handle, p.comment_count, p.hidden, p.resolved, p.created_at
         FROM posts p JOIN users u ON u.id = p.user_id
         WHERE p.problem_id = $1 AND p.id < $2 AND (NOT p.hidden OR $3)
         ORDER BY p.id DESC LIMIT $4",
    )
    .bind(id)
    .bind(before)
    .bind(admin)
    .bind(limit + 1)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(page::finish(rows, limit, |r| (0, r.id))))
}

#[derive(Deserialize, ToSchema)]
pub struct PostBody {
    pub kind: String,
    pub title: String,
    pub body: String,
    pub code: Option<String>,
    pub code_language: Option<String>,
}

fn check_code(
    code: &Option<String>,
    lang: &Option<String>,
) -> ApiResult<(Option<String>, Option<String>)> {
    match code
        .as_deref()
        .map(str::trim_end)
        .filter(|c| !c.trim().is_empty())
    {
        None => Ok((None, None)),
        Some(c) => {
            if c.len() > limits::MAX_CODE_BYTES {
                return Err(ApiError::bad_request("코드는 64KB 이하여야 합니다."));
            }
            let l = match lang {
                Some(l) => Some(
                    l.parse::<Language>()
                        .map_err(|_| ApiError::bad_request("지원하지 않는 언어입니다."))?
                        .id()
                        .to_string(),
                ),
                None => None,
            };
            Ok((Some(c.to_string()), l))
        }
    }
}

fn check_text(title: Option<&str>, body: &str) -> ApiResult<()> {
    if let Some(t) = title {
        let n = t.trim().chars().count();
        if n == 0 || n > 100 {
            return Err(ApiError::bad_request("제목은 1~100자여야 합니다."));
        }
    }
    let n = body.trim().chars().count();
    if n == 0 || n > 20_000 {
        return Err(ApiError::bad_request("본문은 1~20000자여야 합니다."));
    }
    Ok(())
}

async fn write_allowed(s: &AppState, u: &User) -> ApiResult<()> {
    if ratelimit::once_per(s, &format!("write:{}", u.id), 60).await? {
        Ok(())
    } else {
        Err(ApiError::too_many(
            "글과 댓글은 1분에 한 번 쓸 수 있습니다.",
        ))
    }
}

#[derive(Serialize, ToSchema)]
pub struct Created {
    pub id: i64,
}

#[utoipa::path(post, path = "/problems/{id}/posts", tag = "board", params(("id" = i32, Path)), request_body = PostBody,
    responses((status = 200, body = Created), (status = 429, body = crate::error::ErrorBody)))]
async fn create_post(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
    Json(b): Json<PostBody>,
) -> ApiResult<Json<Created>> {
    let p = load_problem(&s, id)
        .await?
        .filter(|p| can_see(p, Some(&u)))
        .ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    let kind = b.kind.to_uppercase();
    if !KINDS.contains(&kind.as_str()) {
        return Err(ApiError::bad_request(
            "글 종류는 질문, 반례, 오타 제보 중 하나입니다.",
        ));
    }
    check_text(Some(&b.title), &b.body)?;
    let (code, code_language) = check_code(&b.code, &b.code_language)?;
    write_allowed(&s, &u).await?;
    let pid: i64 = sqlx::query_scalar(
        "INSERT INTO posts (problem_id, user_id, kind, title, body, code, code_language)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(p.id)
    .bind(u.id)
    .bind(&kind)
    .bind(b.title.trim())
    .bind(b.body.trim_end())
    .bind(code)
    .bind(code_language)
    .fetch_one(&s.db)
    .await?;
    if kind == "TYPO" {
        s.alert(&format!(
            "오타 제보: 문제 {} \"{}\" — {}/board/{pid}",
            p.id,
            b.title.trim(),
            s.cfg.web_url
        ))
        .await;
    }
    Ok(Json(Created { id: pid }))
}

#[derive(Serialize, ToSchema)]
pub struct CommentView {
    pub id: i64,
    pub handle: Option<String>,
    pub body: String,
    pub code: Option<String>,
    pub code_language: Option<String>,
    /// Code exists but the viewer has not solved the problem.
    pub code_hidden: bool,
    pub hidden: bool,
    pub is_mine: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct PostView {
    pub id: i64,
    pub problem_id: i32,
    pub problem_title: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub handle: Option<String>,
    pub code: Option<String>,
    pub code_language: Option<String>,
    pub code_hidden: bool,
    pub hidden: bool,
    pub resolved: bool,
    pub is_mine: bool,
    pub created_at: DateTime<Utc>,
    pub comments: Vec<CommentView>,
}

#[derive(FromRow)]
struct PostRow {
    id: i64,
    problem_id: i32,
    problem_title: String,
    kind: String,
    title: String,
    body: String,
    user_id: i64,
    handle: Option<String>,
    code: Option<String>,
    code_language: Option<String>,
    hidden: bool,
    resolved: bool,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct CommentRow {
    id: i64,
    user_id: i64,
    handle: Option<String>,
    body: String,
    code: Option<String>,
    code_language: Option<String>,
    hidden: bool,
    created_at: DateTime<Utc>,
}

#[utoipa::path(get, path = "/posts/{id}", tag = "board", params(("id" = i64, Path)),
    responses((status = 200, body = PostView), (status = 404, body = crate::error::ErrorBody)))]
async fn get_post(
    State(s): State<AppState>,
    Viewer(v): Viewer,
    Path(id): Path<i64>,
) -> ApiResult<Json<PostView>> {
    let p = sqlx::query_as::<_, PostRow>(
        "SELECT p.id, p.problem_id, pr.title AS problem_title, p.kind, p.title, p.body, p.user_id, u.handle,
                p.code, p.code_language, p.hidden, p.resolved, p.created_at
         FROM posts p JOIN users u ON u.id = p.user_id JOIN problems pr ON pr.id = p.problem_id
         WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::not_found("글을 찾을 수 없습니다."))?;
    let admin = v.as_ref().is_some_and(User::is_admin);
    let me = v.as_ref().map(|u| u.id);
    if p.hidden && !admin && me != Some(p.user_id) {
        return Err(ApiError::not_found("숨겨진 글입니다."));
    }
    let solved = match me {
        Some(uid) => sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM user_problem WHERE user_id = $1 AND problem_id = $2 AND state = 'SOLVED')",
        )
        .bind(uid)
        .bind(p.problem_id)
        .fetch_one(&s.db)
        .await?,
        None => false,
    };
    let comments = sqlx::query_as::<_, CommentRow>(
        "SELECT c.id, c.user_id, u.handle, c.body, c.code, c.code_language, c.hidden, c.created_at
         FROM comments c JOIN users u ON u.id = c.user_id WHERE c.post_id = $1 ORDER BY c.id",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;

    // Attached code is for people who solved the problem (and its writer).
    let show = |owner: i64| admin || solved || me == Some(owner);
    let comments = comments
        .into_iter()
        .filter(|c| !c.hidden || admin || me == Some(c.user_id))
        .map(|c| {
            let visible = show(c.user_id);
            CommentView {
                id: c.id,
                handle: c.handle,
                code_hidden: c.code.is_some() && !visible,
                code: c.code.filter(|_| visible),
                code_language: c.code_language,
                body: c.body,
                hidden: c.hidden,
                is_mine: me == Some(c.user_id),
                created_at: c.created_at,
            }
        })
        .collect();
    let visible = show(p.user_id);
    Ok(Json(PostView {
        id: p.id,
        problem_id: p.problem_id,
        problem_title: p.problem_title,
        kind: p.kind,
        title: p.title,
        body: p.body,
        handle: p.handle,
        code_hidden: p.code.is_some() && !visible,
        code: p.code.filter(|_| visible),
        code_language: p.code_language,
        hidden: p.hidden,
        resolved: p.resolved,
        is_mine: me == Some(p.user_id),
        created_at: p.created_at,
        comments,
    }))
}

/// Authors and admins can remove (hide) a post.
#[utoipa::path(delete, path = "/posts/{id}", tag = "board", params(("id" = i64, Path)), responses((status = 204)))]
async fn delete_post(
    State(s): State<AppState>,
    crate::auth::AuthUser(u): crate::auth::AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let n = sqlx::query(
        "UPDATE posts SET hidden = TRUE, updated_at = now() WHERE id = $1 AND (user_id = $2 OR $3)",
    )
    .bind(id)
    .bind(u.id)
    .bind(u.is_admin())
    .execute(&s.db)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(ApiError::not_found("글을 찾을 수 없습니다."));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct CommentBody {
    pub body: String,
    pub code: Option<String>,
    pub code_language: Option<String>,
}

#[utoipa::path(post, path = "/posts/{id}/comments", tag = "board", params(("id" = i64, Path)), request_body = CommentBody,
    responses((status = 200, body = Created), (status = 429, body = crate::error::ErrorBody)))]
async fn create_comment(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i64>,
    Json(b): Json<CommentBody>,
) -> ApiResult<Json<Created>> {
    let hidden: Option<bool> = sqlx::query_scalar("SELECT hidden FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?;
    if hidden.is_none_or(|h| h && !u.is_admin()) {
        return Err(ApiError::not_found("글을 찾을 수 없습니다."));
    }
    check_text(None, &b.body)?;
    let (code, code_language) = check_code(&b.code, &b.code_language)?;
    write_allowed(&s, &u).await?;
    let mut tx = s.db.begin().await?;
    let cid: i64 = sqlx::query_scalar(
        "INSERT INTO comments (post_id, user_id, body, code, code_language) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(id)
    .bind(u.id)
    .bind(b.body.trim_end())
    .bind(code)
    .bind(code_language)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(Created { id: cid }))
}

#[utoipa::path(delete, path = "/comments/{id}", tag = "board", params(("id" = i64, Path)), responses((status = 204)))]
async fn delete_comment(
    State(s): State<AppState>,
    crate::auth::AuthUser(u): crate::auth::AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let n = sqlx::query("UPDATE comments SET hidden = TRUE, updated_at = now() WHERE id = $1 AND (user_id = $2 OR $3)")
        .bind(id)
        .bind(u.id)
        .bind(u.is_admin())
        .execute(&s.db)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::not_found("댓글을 찾을 수 없습니다."));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct ReportBody {
    /// POST or COMMENT
    pub target_type: String,
    pub target_id: i64,
    pub reason: String,
}

/// Hides the target automatically once enough distinct users report it.
#[utoipa::path(post, path = "/reports", tag = "board", request_body = ReportBody, responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn report(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Json(b): Json<ReportBody>,
) -> ApiResult<StatusCode> {
    let table = match b.target_type.as_str() {
        "POST" => "posts",
        "COMMENT" => "comments",
        _ => {
            return Err(ApiError::bad_request(
                "신고 대상은 POST 또는 COMMENT입니다.",
            ))
        }
    };
    let reason = b.reason.trim();
    if reason.is_empty() || reason.chars().count() > 500 {
        return Err(ApiError::bad_request("신고 사유는 1~500자여야 합니다."));
    }
    let exists: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS (SELECT 1 FROM {table} WHERE id = $1)"
    ))
    .bind(b.target_id)
    .fetch_one(&s.db)
    .await?;
    if !exists {
        return Err(ApiError::not_found("신고할 글을 찾을 수 없습니다."));
    }
    let inserted = sqlx::query(
        "INSERT INTO reports (reporter_id, target_type, target_id, reason) VALUES ($1, $2, $3, $4)
         ON CONFLICT DO NOTHING",
    )
    .bind(u.id)
    .bind(&b.target_type)
    .bind(b.target_id)
    .bind(reason)
    .execute(&s.db)
    .await?
    .rows_affected();
    if inserted == 0 {
        return Err(ApiError::conflict("ALREADY_REPORTED", "이미 신고했습니다."));
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM reports WHERE target_type = $1 AND target_id = $2 AND NOT resolved",
    )
    .bind(&b.target_type)
    .bind(b.target_id)
    .fetch_one(&s.db)
    .await?;
    if count >= limits::REPORT_HIDE_THRESHOLD {
        sqlx::query(&format!("UPDATE {table} SET hidden = TRUE WHERE id = $1"))
            .bind(b.target_id)
            .execute(&s.db)
            .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
