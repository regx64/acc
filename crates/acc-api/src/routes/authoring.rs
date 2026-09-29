//! Problem authoring (`/create`, `/my/problems`): drafts, test data upload,
//! reference-solution check and the review request.
//!
//! Admins use the same endpoints; their problems are OFFICIAL and they may
//! edit problems in any state.

use std::io::{Cursor, Read};

use acc_core::queue::Job;
use acc_core::{limits, storage, Language};
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use object_store::path::Path as ObjPath;
use object_store::ObjectStoreExt;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::problems::{Sample, Statement};
use super::submissions::enqueue;
use crate::auth::{ActiveUser, User};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    let upload = OpenApiRouter::new()
        .routes(routes!(upload_testcases))
        .layer(DefaultBodyLimit::max(
            limits::MAX_TESTCASE_BYTES as usize + 8 * 1024 * 1024,
        ));
    OpenApiRouter::new()
        .routes(routes!(list_mine, create_problem))
        .routes(routes!(get_mine, update_problem))
        .routes(routes!(validate))
        .routes(routes!(request_review))
        .routes(routes!(withdraw_review))
        .merge(upload)
}

#[derive(Deserialize, ToSchema)]
pub struct ProblemInput {
    pub title: String,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    /// Author's suggested level 0..=30; an admin sets the final level.
    pub proposed_level: Option<i16>,
    pub statement: Statement,
    pub solution_language: Option<String>,
    pub solution_code: Option<String>,
    /// Required on create: agrees to the problem authoring terms.
    #[serde(default)]
    pub agree_terms: bool,
}

fn validate_input(b: &ProblemInput) -> ApiResult<()> {
    let title = b.title.trim().chars().count();
    if title == 0 || title > 60 {
        return Err(ApiError::bad_request("제목은 1~60자여야 합니다."));
    }
    if !limits::TIME_LIMIT_MS.contains(&(b.time_limit_ms.max(0) as u32)) {
        return Err(ApiError::bad_request(
            "시간 제한은 0.5~10초(500~10000ms)여야 합니다.",
        ));
    }
    if !limits::MEMORY_LIMIT_KB.contains(&((b.memory_limit_mb.max(0) as u32) * 1024)) {
        return Err(ApiError::bad_request("메모리 제한은 32~1024MB여야 합니다."));
    }
    if b.proposed_level.is_some_and(|l| !(0..=30).contains(&l)) {
        return Err(ApiError::bad_request("레벨은 0~30입니다."));
    }
    let st = &b.statement;
    for (name, text, max) in [
        ("문제", &st.legend, 50_000),
        ("입력", &st.input, 20_000),
        ("출력", &st.output, 20_000),
    ] {
        let n = text.trim().chars().count();
        if n == 0 || n > max {
            return Err(ApiError::bad_request(format!(
                "{name} 섹션은 1~{max}자여야 합니다."
            )));
        }
    }
    if st.hint.as_ref().is_some_and(|h| h.chars().count() > 20_000) {
        return Err(ApiError::bad_request("힌트는 20000자 이하여야 합니다."));
    }
    if st.samples.is_empty() || st.samples.len() > 10 {
        return Err(ApiError::bad_request("예제는 1~10개여야 합니다."));
    }
    if st
        .samples
        .iter()
        .any(|s| s.input.len() > 64 * 1024 || s.output.len() > 64 * 1024)
    {
        return Err(ApiError::bad_request("예제 하나는 64KB 이하여야 합니다."));
    }
    if let Some(l) = &b.solution_language {
        l.parse::<Language>()
            .map_err(|_| ApiError::bad_request("정해 언어가 올바르지 않습니다."))?;
    }
    if b.solution_code
        .as_ref()
        .is_some_and(|c| c.len() > limits::MAX_CODE_BYTES)
    {
        return Err(ApiError::bad_request("정해 코드는 64KB 이하여야 합니다."));
    }
    Ok(())
}

fn normalized(st: &Statement) -> Statement {
    let fix = |s: &str| {
        let mut t = s.replace("\r\n", "\n");
        if !t.ends_with('\n') {
            t.push('\n');
        }
        t
    };
    Statement {
        legend: st.legend.trim().to_string(),
        input: st.input.trim().to_string(),
        output: st.output.trim().to_string(),
        hint: st
            .hint
            .as_ref()
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty()),
        samples: st
            .samples
            .iter()
            .map(|s| Sample {
                input: fix(&s.input),
                output: fix(&s.output),
            })
            .collect(),
    }
}

fn batch_id() -> String {
    let mut b = [0u8; 8];
    rand::fill(&mut b);
    hex::encode(b)
}

#[derive(FromRow)]
struct DataCase {
    input_key: String,
    output_key: String,
    input_size: i64,
    output_size: i64,
}

/// Rewrites the problem's test list: samples first (uploaded fresh), then
/// the uploaded data cases.
async fn rebuild_testcases(
    s: &AppState,
    pid: i32,
    samples: &[Sample],
    data: Vec<DataCase>,
) -> ApiResult<()> {
    let batch = batch_id();
    let mut rows: Vec<(bool, DataCase)> = Vec::new();
    for (i, smp) in samples.iter().enumerate() {
        let idx = i as i32 + 1;
        let ik = storage::testcase_key(pid, &batch, idx, "in");
        let ok = storage::testcase_key(pid, &batch, idx, "out");
        s.store
            .put(
                &ObjPath::from(ik.as_str()),
                smp.input.clone().into_bytes().into(),
            )
            .await?;
        s.store
            .put(
                &ObjPath::from(ok.as_str()),
                smp.output.clone().into_bytes().into(),
            )
            .await?;
        rows.push((
            true,
            DataCase {
                input_key: ik,
                output_key: ok,
                input_size: smp.input.len() as i64,
                output_size: smp.output.len() as i64,
            },
        ));
    }
    rows.extend(data.into_iter().map(|d| (false, d)));

    let mut tx = s.db.begin().await?;
    sqlx::query("DELETE FROM testcases WHERE problem_id = $1")
        .bind(pid)
        .execute(&mut *tx)
        .await?;
    for (i, (sample, d)) in rows.iter().enumerate() {
        sqlx::query(
            "INSERT INTO testcases (problem_id, idx, input_key, output_key, input_size, output_size, is_sample)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(pid)
        .bind(i as i32 + 1)
        .bind(&d.input_key)
        .bind(&d.output_key)
        .bind(d.input_size)
        .bind(d.output_size)
        .bind(sample)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "UPDATE problems SET testcase_version = testcase_version + 1,
            validation_status = NULL, validation_message = NULL, validated_at = NULL
         WHERE id = $1",
    )
    .bind(pid)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn data_cases(s: &AppState, pid: i32) -> ApiResult<Vec<DataCase>> {
    Ok(sqlx::query_as::<_, DataCase>(
        "SELECT input_key, output_key, input_size, output_size FROM testcases
         WHERE problem_id = $1 AND NOT is_sample ORDER BY idx",
    )
    .bind(pid)
    .fetch_all(&s.db)
    .await?)
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct MyProblemRow {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub source: String,
    pub level: i16,
    pub proposed_level: Option<i16>,
    pub validation_status: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Last review comment, e.g. the reason for a rejection.
    pub last_review: Option<String>,
}

#[utoipa::path(get, path = "/my/problems", tag = "authoring", responses((status = 200, body = Vec<MyProblemRow>)))]
async fn list_mine(
    State(s): State<AppState>,
    crate::auth::AuthUser(u): crate::auth::AuthUser,
) -> ApiResult<Json<Vec<MyProblemRow>>> {
    let rows = sqlx::query_as::<_, MyProblemRow>(
        "SELECT p.id, p.title, p.status, p.source, p.level, p.proposed_level, p.validation_status, p.created_at,
                (SELECT r.decision || COALESCE(': ' || r.comment, '') FROM problem_reviews r
                  WHERE r.problem_id = p.id ORDER BY r.id DESC LIMIT 1) AS last_review
         FROM problems p WHERE p.author_id = $1 ORDER BY p.id DESC",
    )
    .bind(u.id)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, ToSchema)]
pub struct CreatedProblem {
    pub id: i32,
}

#[utoipa::path(post, path = "/my/problems", tag = "authoring", request_body = ProblemInput,
    responses((status = 200, body = CreatedProblem), (status = 400, body = crate::error::ErrorBody)))]
async fn create_problem(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Json(b): Json<ProblemInput>,
) -> ApiResult<Json<CreatedProblem>> {
    if !b.agree_terms && !u.is_admin() {
        return Err(ApiError::bad_request("출제 약관에 동의해야 합니다."));
    }
    validate_input(&b)?;
    let drafts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM problems WHERE author_id = $1 AND status <> 'PUBLIC'",
    )
    .bind(u.id)
    .fetch_one(&s.db)
    .await?;
    if drafts >= 20 && !u.is_admin() {
        return Err(ApiError::conflict(
            "TOO_MANY_DRAFTS",
            "공개 전 문제는 20개까지 만들 수 있습니다.",
        ));
    }
    let st = normalized(&b.statement);
    let pid: i32 = sqlx::query_scalar(
        "INSERT INTO problems (title, author_id, source, time_limit_ms, memory_limit_kb, proposed_level, statement,
                               solution_language, solution_code)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
    )
    .bind(b.title.trim())
    .bind(u.id)
    .bind(if u.is_admin() { "OFFICIAL" } else { "USER" })
    .bind(b.time_limit_ms)
    .bind(b.memory_limit_mb * 1024)
    .bind(b.proposed_level)
    .bind(sqlx::types::Json(&st))
    .bind(&b.solution_language)
    .bind(&b.solution_code)
    .fetch_one(&s.db)
    .await?;
    rebuild_testcases(&s, pid, &st.samples, vec![]).await?;
    Ok(Json(CreatedProblem { id: pid }))
}

#[derive(FromRow)]
struct OwnedRow {
    author_id: i64,
    status: String,
}

/// The author (while not public) or an admin may edit.
async fn editable(s: &AppState, u: &User, pid: i32) -> ApiResult<OwnedRow> {
    let row = sqlx::query_as::<_, OwnedRow>("SELECT author_id, status FROM problems WHERE id = $1")
        .bind(pid)
        .fetch_optional(&s.db)
        .await?
        .filter(|r| r.author_id == u.id || u.is_admin())
        .ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    if !u.is_admin() && row.status != "DRAFT" {
        return Err(ApiError::conflict(
            "NOT_EDITABLE",
            "검수 중이거나 공개된 문제는 수정할 수 없습니다. 검수 요청을 취소하거나 관리자에게 문의하세요.",
        ));
    }
    Ok(row)
}

#[derive(Serialize, ToSchema)]
pub struct TestcaseInfo {
    pub idx: i32,
    pub is_sample: bool,
    pub input_size: i64,
    pub output_size: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ReviewEntry {
    pub decision: String,
    pub comment: Option<String>,
    pub reviewer: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct MyProblem {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub source: String,
    pub level: i16,
    pub proposed_level: Option<i16>,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub statement: Statement,
    pub solution_language: Option<String>,
    pub solution_code: Option<String>,
    pub validation_status: Option<String>,
    pub validation_message: Option<String>,
    pub validated_at: Option<DateTime<Utc>>,
    pub testcases: Vec<TestcaseInfo>,
    pub reviews: Vec<ReviewEntry>,
}

#[derive(FromRow)]
struct MyProblemDb {
    id: i32,
    title: String,
    status: String,
    source: String,
    level: i16,
    proposed_level: Option<i16>,
    time_limit_ms: i32,
    memory_limit_kb: i32,
    statement: sqlx::types::Json<Statement>,
    solution_language: Option<String>,
    solution_code: Option<String>,
    validation_status: Option<String>,
    validation_message: Option<String>,
    validated_at: Option<DateTime<Utc>>,
    author_id: i64,
}

#[utoipa::path(get, path = "/my/problems/{id}", tag = "authoring", params(("id" = i32, Path)),
    responses((status = 200, body = MyProblem), (status = 404, body = crate::error::ErrorBody)))]
async fn get_mine(
    State(s): State<AppState>,
    crate::auth::AuthUser(u): crate::auth::AuthUser,
    Path(id): Path<i32>,
) -> ApiResult<Json<MyProblem>> {
    let p = sqlx::query_as::<_, MyProblemDb>(
        "SELECT id, title, status, source, level, proposed_level, time_limit_ms, memory_limit_kb, statement,
                solution_language, solution_code, validation_status, validation_message, validated_at, author_id
         FROM problems WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?
    .filter(|p| p.author_id == u.id || u.is_admin())
    .ok_or_else(|| ApiError::not_found("문제를 찾을 수 없습니다."))?;
    let testcases: Vec<(i32, bool, i64, i64)> = sqlx::query_as(
        "SELECT idx, is_sample, input_size, output_size FROM testcases WHERE problem_id = $1 ORDER BY idx",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    type ReviewTuple = (String, Option<String>, Option<String>, DateTime<Utc>);
    let reviews: Vec<ReviewTuple> = sqlx::query_as(
        "SELECT r.decision, r.comment, u.handle, r.created_at FROM problem_reviews r
         LEFT JOIN users u ON u.id = r.reviewer_id WHERE r.problem_id = $1 ORDER BY r.id",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(MyProblem {
        id: p.id,
        title: p.title,
        status: p.status,
        source: p.source,
        level: p.level,
        proposed_level: p.proposed_level,
        time_limit_ms: p.time_limit_ms,
        memory_limit_mb: p.memory_limit_kb / 1024,
        statement: p.statement.0,
        solution_language: p.solution_language,
        solution_code: p.solution_code,
        validation_status: p.validation_status,
        validation_message: p.validation_message,
        validated_at: p.validated_at,
        testcases: testcases
            .into_iter()
            .map(|(idx, is_sample, input_size, output_size)| TestcaseInfo {
                idx,
                is_sample,
                input_size,
                output_size,
            })
            .collect(),
        reviews: reviews
            .into_iter()
            .map(|(decision, comment, reviewer, created_at)| ReviewEntry {
                decision,
                comment,
                reviewer,
                created_at,
            })
            .collect(),
    }))
}

#[utoipa::path(put, path = "/my/problems/{id}", tag = "authoring", params(("id" = i32, Path)), request_body = ProblemInput,
    responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn update_problem(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
    Json(b): Json<ProblemInput>,
) -> ApiResult<StatusCode> {
    editable(&s, &u, id).await?;
    validate_input(&b)?;
    let st = normalized(&b.statement);
    let old: sqlx::types::Json<Statement> =
        sqlx::query_scalar("SELECT statement FROM problems WHERE id = $1")
            .bind(id)
            .fetch_one(&s.db)
            .await?;
    // Anything that can change the verdict of the reference solution resets the check.
    let changed: bool = sqlx::query_scalar(
        "UPDATE problems p SET title = $2, time_limit_ms = $3, memory_limit_kb = $4, proposed_level = $5,
                statement = $6, solution_language = $7, solution_code = $8
         FROM (SELECT time_limit_ms, memory_limit_kb, solution_language, solution_code FROM problems WHERE id = $1) old
         WHERE p.id = $1
         RETURNING (old.time_limit_ms <> $3 OR old.memory_limit_kb <> $4
                    OR old.solution_language IS DISTINCT FROM $7 OR old.solution_code IS DISTINCT FROM $8)",
    )
    .bind(id)
    .bind(b.title.trim())
    .bind(b.time_limit_ms)
    .bind(b.memory_limit_mb * 1024)
    .bind(b.proposed_level)
    .bind(sqlx::types::Json(&st))
    .bind(&b.solution_language)
    .bind(&b.solution_code)
    .fetch_one(&s.db)
    .await?;
    let samples_changed = old.0.samples.len() != st.samples.len()
        || old
            .0
            .samples
            .iter()
            .zip(&st.samples)
            .any(|(a, b)| a.input != b.input || a.output != b.output);
    if samples_changed {
        let data = data_cases(&s, id).await?;
        rebuild_testcases(&s, id, &st.samples, data).await?;
    } else if changed {
        sqlx::query("UPDATE problems SET validation_status = NULL, validation_message = NULL, validated_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&s.db)
            .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(ToSchema)]
#[allow(dead_code)]
pub struct TestcaseUpload {
    /// Zip of `1.in`, `1.out`, `2.in`, `2.out`, ... (samples are added automatically).
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

/// Reads `n.in` / `n.out` pairs from a zip, numbered 1..=k without gaps.
fn read_zip(bytes: &[u8]) -> ApiResult<Vec<(Vec<u8>, Vec<u8>)>> {
    let bad = |m: &str| ApiError::bad_request(m.to_string());
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| bad("zip 파일을 읽을 수 없습니다."))?;
    type Pair = (Option<Vec<u8>>, Option<Vec<u8>>);
    let mut files: std::collections::BTreeMap<u32, Pair> = Default::default();
    let mut total: u64 = 0;
    for i in 0..zip.len() {
        let mut f = zip
            .by_index(i)
            .map_err(|_| bad("zip 파일을 읽을 수 없습니다."))?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().rsplit('/').next().unwrap_or_default().to_string();
        if name.starts_with('.') || f.name().starts_with("__MACOSX") {
            continue;
        }
        let (stem, ext) = name
            .rsplit_once('.')
            .ok_or_else(|| bad(&format!("알 수 없는 파일: {name}")))?;
        let n: u32 = stem.parse().map_err(|_| {
            bad(&format!(
                "파일 이름은 숫자.in / 숫자.out 이어야 합니다: {name}"
            ))
        })?;
        let mut buf = Vec::new();
        // Never trust the declared size: cap what is actually read.
        let remaining = limits::MAX_TESTCASE_BYTES - total;
        (&mut f)
            .take(remaining + 1)
            .read_to_end(&mut buf)
            .map_err(|_| bad("zip 파일을 읽을 수 없습니다."))?;
        total += buf.len() as u64;
        if total > limits::MAX_TESTCASE_BYTES {
            return Err(bad("테스트 데이터는 모두 합쳐 50MB 이하여야 합니다."));
        }
        let entry = files.entry(n).or_default();
        match ext {
            "in" => entry.0 = Some(buf),
            "out" => entry.1 = Some(buf),
            _ => return Err(bad(&format!("알 수 없는 파일: {name}"))),
        }
    }
    if files.is_empty() {
        return Err(bad("zip 안에 n.in / n.out 파일이 없습니다."));
    }
    if files.len() > limits::MAX_TESTCASES {
        return Err(bad("테스트는 문제당 100개 이하여야 합니다(예제 제외)."));
    }
    let mut out = Vec::with_capacity(files.len());
    for (expect, (n, (i, o))) in (1u32..).zip(files) {
        if n != expect {
            return Err(bad(&format!(
                "{expect}번 테스트가 없습니다. 번호는 1부터 빠짐없이 이어져야 합니다."
            )));
        }
        match (i, o) {
            (Some(i), Some(o)) => out.push((i, o)),
            _ => return Err(bad(&format!("{n}.in과 {n}.out이 짝을 이루어야 합니다."))),
        }
    }
    Ok(out)
}

#[utoipa::path(put, path = "/my/problems/{id}/testcases", tag = "authoring", params(("id" = i32, Path)),
    request_body(content = TestcaseUpload, content_type = "multipart/form-data"),
    responses((status = 204), (status = 400, body = crate::error::ErrorBody)))]
async fn upload_testcases(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
    mut form: Multipart,
) -> ApiResult<StatusCode> {
    editable(&s, &u, id).await?;
    let mut bytes = None;
    while let Some(field) = form
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?
    {
        if field.name() == Some("file") {
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::bad_request(e.to_string()))?,
            );
        }
    }
    let bytes = bytes.ok_or_else(|| ApiError::bad_request("file 필드가 없습니다."))?;
    let pairs = read_zip(&bytes)?;

    let batch = batch_id();
    let mut data = Vec::with_capacity(pairs.len());
    for (i, (inp, out)) in pairs.into_iter().enumerate() {
        // Data cases follow the samples; keys only need to be unique.
        let idx = 100 + i as i32;
        let ik = storage::testcase_key(id, &batch, idx, "in");
        let ok = storage::testcase_key(id, &batch, idx, "out");
        let (is, os) = (inp.len() as i64, out.len() as i64);
        s.store.put(&ObjPath::from(ik.as_str()), inp.into()).await?;
        s.store.put(&ObjPath::from(ok.as_str()), out.into()).await?;
        data.push(DataCase {
            input_key: ik,
            output_key: ok,
            input_size: is,
            output_size: os,
        });
    }
    let st: sqlx::types::Json<Statement> =
        sqlx::query_scalar("SELECT statement FROM problems WHERE id = $1")
            .bind(id)
            .fetch_one(&s.db)
            .await?;
    rebuild_testcases(&s, id, &st.0.samples, data).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Queues the reference-solution check: every case must pass within half
/// the time limit.
#[utoipa::path(post, path = "/my/problems/{id}/validate", tag = "authoring", params(("id" = i32, Path)),
    responses((status = 204), (status = 400, body = crate::error::ErrorBody)))]
async fn validate(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    editable(&s, &u, id).await?;
    let (has_solution, data_count): (bool, i64) = sqlx::query_as(
        "SELECT solution_code IS NOT NULL AND solution_language IS NOT NULL,
                (SELECT count(*) FROM testcases WHERE problem_id = $1 AND NOT is_sample)
         FROM problems WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    if !has_solution {
        return Err(ApiError::bad_request("정해 코드와 언어를 먼저 저장하세요."));
    }
    if data_count == 0 {
        return Err(ApiError::bad_request("테스트 데이터 zip을 먼저 올리세요."));
    }
    sqlx::query("UPDATE problems SET validation_status = 'PENDING', validation_message = NULL WHERE id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    enqueue(&s, Job::Validate(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/my/problems/{id}/request-review", tag = "authoring", params(("id" = i32, Path)),
    responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn request_review(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    let row = editable(&s, &u, id).await?;
    if row.status != "DRAFT" {
        return Err(ApiError::conflict(
            "NOT_DRAFT",
            "초안 상태에서만 검수를 요청할 수 있습니다.",
        ));
    }
    let passed: Option<String> =
        sqlx::query_scalar("SELECT validation_status FROM problems WHERE id = $1")
            .bind(id)
            .fetch_one(&s.db)
            .await?;
    if passed.as_deref() != Some("PASSED") {
        return Err(ApiError::conflict(
            "NOT_VALIDATED",
            "정해 검증을 통과해야 검수를 요청할 수 있습니다.",
        ));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("UPDATE problems SET status = 'REVIEW' WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO problem_reviews (problem_id, reviewer_id, decision) VALUES ($1, NULL, 'SUBMITTED')")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    s.alert(&format!("검수 요청: 문제 {id} ({})", u.handle()))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/my/problems/{id}/withdraw", tag = "authoring", params(("id" = i32, Path)),
    responses((status = 204), (status = 409, body = crate::error::ErrorBody)))]
async fn withdraw_review(
    State(s): State<AppState>,
    ActiveUser(u): ActiveUser,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    let n = sqlx::query("UPDATE problems SET status = 'DRAFT' WHERE id = $1 AND author_id = $2 AND status = 'REVIEW'")
        .bind(id)
        .bind(u.id)
        .execute(&s.db)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::conflict(
            "NOT_IN_REVIEW",
            "검수 중인 문제가 아닙니다.",
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::read_zip;

    fn zip(files: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut z = zip::ZipWriter::new(&mut buf);
        for (name, body) in files {
            z.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
        buf.into_inner()
    }

    #[test]
    fn pairs_in_order() {
        let z = zip(&[
            ("2.in", "b"),
            ("1.in", "a"),
            ("1.out", "A"),
            ("data/2.out", "B"),
        ]);
        let got = read_zip(&z).unwrap();
        assert_eq!(
            got,
            vec![
                (b"a".to_vec(), b"A".to_vec()),
                (b"b".to_vec(), b"B".to_vec())
            ]
        );
    }

    #[test]
    fn rejects_gaps_and_orphans() {
        assert!(read_zip(&zip(&[
            ("1.in", "a"),
            ("1.out", "A"),
            ("3.in", "c"),
            ("3.out", "C")
        ]))
        .is_err());
        assert!(read_zip(&zip(&[("1.in", "a")])).is_err());
        assert!(read_zip(&zip(&[("x.txt", "a")])).is_err());
        assert!(read_zip(b"not a zip").is_err());
    }
}
