use std::path::PathBuf;

use acc_judge::gojudge::GoJudgeConfig;
use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub storage_url: String,
    pub judge: GoJudgeConfig,
    /// Local test data cache.
    pub cache_dir: PathBuf,
    /// The same directory as go-judge sees it. When set, inputs are passed
    /// by path instead of being sent inline.
    pub judge_cache_dir: Option<PathBuf>,
    pub worker_id: String,
    pub alert_webhook: Option<String>,
}

fn var(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("{name} is not set"))
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: var("DATABASE_URL")?,
            redis_url: var("REDIS_URL")?,
            storage_url: var("STORAGE_URL")?,
            judge: GoJudgeConfig::from_env(),
            cache_dir: std::env::var("CACHE_DIR")
                .unwrap_or_else(|_| "/var/cache/acc".into())
                .into(),
            judge_cache_dir: std::env::var("JUDGE_CACHE_DIR")
                .ok()
                .filter(|s| !s.is_empty())
                .map(Into::into),
            worker_id: std::env::var("WORKER_ID").unwrap_or_else(|_| "worker-1".into()),
            alert_webhook: std::env::var("ALERT_WEBHOOK_URL")
                .ok()
                .filter(|s| !s.is_empty()),
        })
    }
}
