//! Judge worker: pops jobs from Redis, judges them with go-judge, writes
//! results back to Postgres.
//!
//! On SIGTERM it finishes the job in hand and exits, so deploys never cut a
//! judgement in half.

mod cache;
mod config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use acc_core::queue::{self, Job};
use acc_core::{db as derived, Language, Status};
use acc_judge::gojudge::GoJudge;
use acc_judge::{Judge, JudgeOutcome, JudgeRequest};
use anyhow::Context;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Direction};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::{error, info, warn};

use crate::cache::TestcaseCache;
use crate::config::Config;

struct Worker {
    cfg: Config,
    db: PgPool,
    redis_client: redis::Client,
    judge: GoJudge,
    cache: TestcaseCache,
    http: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(cfg.concurrency as u32 + 2)
        .connect(&cfg.database_url)
        .await
        .context("connecting to postgres")?;
    let redis_client = redis::Client::open(cfg.redis_url.as_str())?;
    let store = acc_core::storage::from_url(&cfg.storage_url)?;
    let judge = GoJudge::new(cfg.judge.clone());
    wait_for_judge(&judge).await;

    let cache = TestcaseCache::new(cfg.cache_dir.clone(), cfg.judge_cache_dir.clone(), store);
    let worker = Arc::new(Worker {
        cfg,
        db,
        redis_client,
        judge,
        cache,
        http: reqwest::Client::new(),
    });

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            info!("shutdown requested; finishing the current job");
            stop.store(true, Ordering::SeqCst);
        });
    }

    // Each slot judges one submission at a time on its own Redis connection
    // (BLMOVE blocks the connection it runs on) and its own processing list.
    let mut slots = Vec::new();
    for slot in 0..worker.cfg.concurrency {
        let conn = worker.connect().await?;
        worker.requeue_orphans(conn.clone(), slot).await?;
        let (w, stop) = (worker.clone(), stop.clone());
        slots.push(tokio::spawn(async move { w.run(conn, slot, stop).await }));
    }
    info!(worker = %worker.cfg.worker_id, slots = worker.cfg.concurrency, "worker started");
    for s in slots {
        let _ = s.await;
    }
    info!("worker stopped");
    Ok(())
}

async fn wait_for_judge(judge: &GoJudge) {
    loop {
        match judge.version().await {
            Ok(v) => {
                info!(version = %v, "go-judge ready");
                return;
            }
            Err(e) => {
                warn!("waiting for go-judge: {e:#}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("signal handler");
        tokio::select! {
            _ = ctrl_c => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    let _ = ctrl_c.await;
}

impl Worker {
    fn processing_key(&self, slot: usize) -> String {
        queue::processing(&format!("{}-{slot}", self.cfg.worker_id))
    }

    async fn connect(&self) -> anyhow::Result<ConnectionManager> {
        // BLMOVE blocks for a while, so the response timeout must be longer.
        self.redis_client
            .get_connection_manager_with_config(
                redis::aio::ConnectionManagerConfig::new()
                    .set_response_timeout(Some(Duration::from_secs(10))),
            )
            .await
            .context("connecting to redis")
    }

    /// Puts back jobs this slot held when the worker last died.
    async fn requeue_orphans(&self, mut r: ConnectionManager, slot: usize) -> anyhow::Result<()> {
        loop {
            let moved: Option<String> = r
                .lmove(
                    self.processing_key(slot),
                    queue::QUEUE,
                    Direction::Right,
                    Direction::Right,
                )
                .await?;
            match moved {
                Some(job) => warn!(%job, "requeued unfinished job"),
                None => return Ok(()),
            }
        }
    }

    async fn run(&self, mut r: ConnectionManager, slot: usize, stop: Arc<AtomicBool>) {
        while !stop.load(Ordering::SeqCst) {
            let popped: redis::RedisResult<Option<String>> = r
                .blmove(
                    queue::QUEUE,
                    self.processing_key(slot),
                    Direction::Right,
                    Direction::Left,
                    2.0,
                )
                .await;
            let raw = match popped {
                Ok(Some(raw)) => raw,
                Ok(None) => continue,
                Err(e) => {
                    error!("redis: {e}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };
            match Job::decode(&raw) {
                Some(Job::Submission(id)) => {
                    if let Err(e) = self.judge_submission(id).await {
                        error!(submission = id, "judging failed: {e:#}");
                        self.mark_system_error(id, &format!("{e:#}")).await;
                    }
                }
                Some(Job::Validate(pid)) => {
                    if let Err(e) = self.validate_problem(pid).await {
                        error!(problem = pid, "validation failed: {e:#}");
                        let _ = sqlx::query(
                            "UPDATE problems SET validation_status = 'FAILED', validation_message = $2, validated_at = now() WHERE id = $1",
                        )
                        .bind(pid)
                        .bind("채점 서버 오류로 검증하지 못했습니다. 잠시 후 다시 시도하세요.")
                        .execute(&self.db)
                        .await;
                    }
                }
                None => warn!(%raw, "dropping malformed job"),
            }
            let _: redis::RedisResult<usize> = r.lrem(self.processing_key(slot), 1, &raw).await;
        }
    }

    async fn load_request(
        &self,
        problem_id: i32,
        language: Language,
        code: String,
    ) -> anyhow::Result<Option<JudgeRequest>> {
        let p = sqlx::query("SELECT time_limit_ms, memory_limit_kb FROM problems WHERE id = $1")
            .bind(problem_id)
            .fetch_one(&self.db)
            .await?;
        let rows = sqlx::query(
            "SELECT input_key, output_key FROM testcases WHERE problem_id = $1 ORDER BY idx",
        )
        .bind(problem_id)
        .fetch_all(&self.db)
        .await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let mut cases = Vec::with_capacity(rows.len());
        for row in rows {
            cases.push(
                self.cache
                    .get(row.get("input_key"), row.get("output_key"))
                    .await?,
            );
        }
        Ok(Some(JudgeRequest {
            language,
            code,
            time_limit_ms: p.get::<i32, _>("time_limit_ms") as u64,
            memory_limit_kb: p.get::<i32, _>("memory_limit_kb") as u64,
            cases,
        }))
    }

    async fn judge_submission(&self, id: i64) -> anyhow::Result<()> {
        let Some(row) = sqlx::query(
            "UPDATE submissions SET status = 'JUDGING' WHERE id = $1 AND status IN ('PENDING', 'JUDGING')
             RETURNING user_id, problem_id, language, code",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        else {
            warn!(submission = id, "not pending; skipped");
            return Ok(());
        };
        let user_id: i64 = row.get("user_id");
        let problem_id: i32 = row.get("problem_id");
        let language: Language = row
            .get::<String, _>("language")
            .parse()
            .map_err(anyhow::Error::msg)?;

        let outcome = match self
            .load_request(problem_id, language, row.get("code"))
            .await?
        {
            Some(req) => self.judge.judge(&req).await,
            None => JudgeOutcome::system_error("problem has no test data"),
        };

        // Compile output is shown to the author; SE detail stays internal.
        let message = match outcome.status {
            Status::Ce | Status::Se => outcome.message.clone(),
            _ => None,
        };
        sqlx::query(
            "UPDATE submissions SET status = $2, time_ms = $3, memory_kb = $4, message = $5, judged_at = now()
             WHERE id = $1",
        )
        .bind(id)
        .bind(outcome.status.as_str())
        .bind(outcome.time_ms as i32)
        .bind(outcome.memory_kb as i32)
        .bind(message)
        .execute(&self.db)
        .await?;

        if derived::refresh_user_problem(&self.db, user_id, problem_id).await? {
            derived::recompute_rating(&self.db, user_id).await?;
        }
        info!(
            submission = id,
            problem = problem_id,
            status = %outcome.status,
            time_ms = outcome.time_ms,
            failed_case = ?outcome.failed_case,
            "judged"
        );
        if outcome.status == Status::Se {
            self.alert(&format!(
                "채점 오류(SE): 제출 {id}, 문제 {problem_id}: {}",
                outcome.message.as_deref().unwrap_or("-")
            ))
            .await;
        }
        Ok(())
    }

    async fn mark_system_error(&self, id: i64, detail: &str) {
        let res = sqlx::query(
            "UPDATE submissions SET status = 'SE', message = $2, judged_at = now() WHERE id = $1 RETURNING user_id, problem_id",
        )
        .bind(id)
        .bind(detail)
        .fetch_optional(&self.db)
        .await;
        if let Ok(Some(row)) = res {
            let _ =
                derived::refresh_user_problem(&self.db, row.get("user_id"), row.get("problem_id"))
                    .await;
        }
        self.alert(&format!("채점 오류(SE): 제출 {id}: {detail}"))
            .await;
    }

    /// Runs the reference solution: every case must pass within half the
    /// time limit.
    async fn validate_problem(&self, problem_id: i32) -> anyhow::Result<()> {
        let row =
            sqlx::query("SELECT solution_language, solution_code FROM problems WHERE id = $1")
                .bind(problem_id)
                .fetch_one(&self.db)
                .await?;
        let (Some(lang), Some(code)) = (
            row.get::<Option<String>, _>("solution_language"),
            row.get::<Option<String>, _>("solution_code"),
        ) else {
            return self
                .set_validation(problem_id, false, "정해가 없습니다.")
                .await;
        };
        let language: Language = lang.parse().map_err(anyhow::Error::msg)?;
        let Some(req) = self.load_request(problem_id, language, code).await? else {
            return self
                .set_validation(problem_id, false, "테스트 데이터가 없습니다.")
                .await;
        };
        let half = language.adjusted_time_ms(req.time_limit_ms) / 2;
        let out = self.judge.judge(&req).await;
        let (ok, msg) = match out.status {
            Status::Ac if u64::from(out.time_ms) <= half => (
                true,
                format!(
                    "통과: {}개 케이스, 최대 {}ms / {}KB",
                    out.passed, out.time_ms, out.memory_kb
                ),
            ),
            Status::Ac => (
                false,
                format!(
                    "정해가 {}ms 걸렸습니다. 시간 제한의 절반({}ms) 이내여야 합니다.",
                    out.time_ms, half
                ),
            ),
            Status::Ce => (
                false,
                format!("정해 컴파일 에러:\n{}", out.message.unwrap_or_default()),
            ),
            Status::Se => (false, "채점 서버 오류로 검증하지 못했습니다.".to_string()),
            s => (
                false,
                // The author owns the data, so the case number is shown here.
                format!(
                    "정해가 {}번 케이스에서 {} 판정을 받았습니다.",
                    out.failed_case.unwrap_or(0),
                    s
                ),
            ),
        };
        info!(problem = problem_id, ok, "validated");
        self.set_validation(problem_id, ok, &msg).await
    }

    async fn set_validation(&self, problem_id: i32, ok: bool, msg: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE problems SET validation_status = $2, validation_message = $3, validated_at = now() WHERE id = $1",
        )
        .bind(problem_id)
        .bind(if ok { "PASSED" } else { "FAILED" })
        .bind(msg)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Sends an operator alert to the configured Discord-compatible webhook.
    async fn alert(&self, text: &str) {
        let Some(url) = &self.cfg.alert_webhook else {
            return;
        };
        let text: String = text.chars().take(1800).collect();
        if let Err(e) = self
            .http
            .post(url)
            .json(&serde_json::json!({ "content": format!("[acc-worker {}] {text}", self.cfg.worker_id) }))
            .send()
            .await
        {
            warn!("alert webhook failed: {e}");
        }
    }
}
