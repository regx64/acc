//! [go-judge](https://github.com/criyle/go-judge) REST backend.

use std::collections::HashMap;
use std::time::Duration;

use acc_core::{compare::outputs_match, Status};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::lang::{self, Recipe};
use crate::{Input, Judge, JudgeOutcome, JudgeRequest};

const STDERR_MAX: u64 = 64 * 1024;
const COMPILE_OUTPUT_MAX: u64 = 16 * 1024;
const MESSAGE_MAX_CHARS: usize = 8 * 1024;
const STDOUT_HARD_MAX: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct GoJudgeConfig {
    /// Base URL, e.g. `http://127.0.0.1:5050`.
    pub url: String,
    /// `PATH` inside the sandbox.
    pub path_env: String,
    /// Extra `KEY=VALUE` variables inside the sandbox.
    pub extra_env: Vec<String>,
}

impl Default for GoJudgeConfig {
    fn default() -> Self {
        GoJudgeConfig {
            url: "http://127.0.0.1:5050".into(),
            path_env: "/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin".into(),
            extra_env: vec![],
        }
    }
}

impl GoJudgeConfig {
    /// Reads `GO_JUDGE_URL`, `GO_JUDGE_PATH` and `GO_JUDGE_ENV` (comma separated).
    pub fn from_env() -> Self {
        let mut c = GoJudgeConfig::default();
        if let Ok(v) = std::env::var("GO_JUDGE_URL") {
            c.url = v;
        }
        if let Ok(v) = std::env::var("GO_JUDGE_PATH") {
            c.path_env = v;
        }
        if let Ok(v) = std::env::var("GO_JUDGE_ENV") {
            c.extra_env = v
                .split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        c
    }
}

pub struct GoJudge {
    cfg: GoJudgeConfig,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunResult {
    status: String,
    #[serde(default)]
    exit_status: i64,
    #[serde(default)]
    error: Option<String>,
    /// CPU time in ns.
    #[serde(default)]
    time: u64,
    /// Peak memory in bytes.
    #[serde(default)]
    memory: u64,
    #[serde(default)]
    files: HashMap<String, String>,
    #[serde(default)]
    file_ids: HashMap<String, String>,
}

#[derive(Serialize)]
struct RunBody {
    cmd: Vec<Value>,
}

impl GoJudge {
    pub fn new(cfg: GoJudgeConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("http client");
        GoJudge { cfg, http }
    }

    fn env(&self) -> Vec<String> {
        let mut env = vec![
            format!("PATH={}", self.cfg.path_env),
            "HOME=/w".into(),
            "LANG=C.UTF-8".into(),
            "PYTHONIOENCODING=utf-8".into(),
        ];
        env.extend(self.cfg.extra_env.iter().cloned());
        env
    }

    async fn run_one(&self, cmd: Value) -> anyhow::Result<RunResult> {
        let res = self
            .http
            .post(format!("{}/run", self.cfg.url))
            .json(&RunBody { cmd: vec![cmd] })
            .send()
            .await?
            .error_for_status()?;
        let mut out: Vec<RunResult> = res.json().await?;
        out.pop()
            .ok_or_else(|| anyhow::anyhow!("empty go-judge response"))
    }

    async fn delete_files(&self, ids: impl IntoIterator<Item = String>) {
        for id in ids {
            let _ = self
                .http
                .delete(format!("{}/file/{id}", self.cfg.url))
                .send()
                .await;
        }
    }

    /// Health check used by the worker at startup.
    pub async fn version(&self) -> anyhow::Result<Value> {
        Ok(self
            .http
            .get(format!("{}/version", self.cfg.url))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    async fn judge_inner(
        &self,
        req: &JudgeRequest,
        cached: &mut Vec<String>,
    ) -> anyhow::Result<JudgeOutcome> {
        let recipe: Recipe = lang::recipe(req.language, req.memory_limit_kb);
        let src_name = req.language.source_file();

        // Compile once; keep artifacts in go-judge's file store.
        let mut artifacts: HashMap<String, String> = HashMap::new();
        if let Some(compile) = &recipe.compile {
            let cmd = json!({
                "args": compile,
                "env": self.env(),
                "files": [
                    {"content": ""},
                    {"name": "stdout", "max": COMPILE_OUTPUT_MAX},
                    {"name": "stderr", "max": COMPILE_OUTPUT_MAX},
                ],
                "cpuLimit": ms_to_ns(lang::COMPILE_TIME_MS),
                "clockLimit": ms_to_ns(lang::COMPILE_TIME_MS * 2),
                "memoryLimit": kb_to_bytes(lang::COMPILE_MEMORY_KB),
                "procLimit": lang::COMPILE_PROC_LIMIT,
                "copyIn": { src_name: {"content": req.code} },
                "copyOut": ["stdout", "stderr"],
                "copyOutCached": recipe.artifacts,
            });
            let r = self.run_one(cmd).await?;
            cached.extend(r.file_ids.values().cloned());
            if r.status == "Internal Error" || r.status == "File Error" {
                return Ok(JudgeOutcome::system_error(format!(
                    "compile: {} {}",
                    r.status,
                    r.error.unwrap_or_default()
                )));
            }
            if r.status != "Accepted" || r.exit_status != 0 {
                let mut msg = String::new();
                for k in ["stderr", "stdout"] {
                    if let Some(s) = r.files.get(k) {
                        msg.push_str(s);
                    }
                }
                if r.status != "Accepted" && r.status != "Nonzero Exit Status" {
                    msg.push_str(&format!("\n[compiler: {}]", r.status));
                }
                return Ok(JudgeOutcome {
                    status: Status::Ce,
                    time_ms: 0,
                    memory_kb: 0,
                    message: Some(clean_message(&msg)),
                    passed: 0,
                    failed_case: None,
                });
            }
            for name in recipe.artifacts {
                match r.file_ids.get(*name) {
                    Some(id) => {
                        artifacts.insert((*name).to_string(), id.clone());
                    }
                    None => {
                        return Ok(JudgeOutcome::system_error(format!(
                            "missing artifact {name}"
                        )))
                    }
                }
            }
        }

        let mut copy_in = serde_json::Map::new();
        for (name, id) in &artifacts {
            copy_in.insert(name.clone(), json!({"fileId": id}));
        }
        if recipe.run_needs_source {
            copy_in.insert(src_name.into(), json!({"content": req.code}));
        }

        let time_ms = req.language.adjusted_time_ms(req.time_limit_ms);
        let mem_bytes = kb_to_bytes(req.memory_limit_kb + recipe.memory_overhead_kb);
        let mut max_time = 0u32;
        let mut max_mem = 0u32;

        for (i, case) in req.cases.iter().enumerate() {
            let stdin = match &case.input {
                Input::Path(p) => json!({"src": p}),
                Input::Bytes(b) => json!({"content": String::from_utf8_lossy(b)}),
            };
            let stdout_max = (case.expected.len() as u64 * 2 + 1024 * 1024).min(STDOUT_HARD_MAX);
            let cmd = json!({
                "args": recipe.run,
                "env": self.env(),
                "files": [
                    stdin,
                    {"name": "stdout", "max": stdout_max},
                    {"name": "stderr", "max": STDERR_MAX},
                ],
                "cpuLimit": ms_to_ns(time_ms),
                "clockLimit": ms_to_ns(time_ms * 2 + 1000),
                "memoryLimit": mem_bytes,
                "stackLimit": mem_bytes,
                "procLimit": recipe.proc_limit,
                "copyIn": copy_in,
                "copyOut": ["stdout"],
            });
            let r = self.run_one(cmd).await?;
            let t = (r.time / 1_000_000) as u32;
            let m = (r.memory / 1024) as u32;
            max_time = max_time.max(t.min(time_ms as u32));
            max_mem = max_mem.max(m);

            let status = match r.status.as_str() {
                "Accepted" => {
                    let out = r.files.get("stdout").map(String::as_bytes).unwrap_or(b"");
                    if outputs_match(&case.expected, out) {
                        None
                    } else {
                        Some(Status::Wa)
                    }
                }
                "Time Limit Exceeded" => Some(Status::Tle),
                "Memory Limit Exceeded" => Some(Status::Mle),
                // Output far larger than the answer cannot be correct.
                "Output Limit Exceeded" => Some(Status::Wa),
                "Nonzero Exit Status" | "Signalled" => Some(runtime_status(req, &r)),
                other => {
                    return Ok(JudgeOutcome::system_error(format!(
                        "case {}: {} {}",
                        i + 1,
                        other,
                        r.error.unwrap_or_default()
                    )))
                }
            };
            if let Some(status) = status {
                return Ok(JudgeOutcome {
                    status,
                    time_ms: max_time,
                    memory_kb: max_mem,
                    message: None,
                    passed: i,
                    failed_case: Some(i + 1),
                });
            }
        }

        Ok(JudgeOutcome {
            status: Status::Ac,
            time_ms: max_time,
            memory_kb: max_mem,
            message: None,
            passed: req.cases.len(),
            failed_case: None,
        })
    }
}

/// A crash near the memory limit is almost always an allocation failure.
fn runtime_status(req: &JudgeRequest, r: &RunResult) -> Status {
    let used_kb = r.memory / 1024;
    if used_kb >= req.memory_limit_kb {
        Status::Mle
    } else {
        Status::Re
    }
}

#[async_trait]
impl Judge for GoJudge {
    async fn judge(&self, req: &JudgeRequest) -> JudgeOutcome {
        let mut cached = Vec::new();
        let out = match self.judge_inner(req, &mut cached).await {
            Ok(o) => o,
            Err(e) => JudgeOutcome::system_error(format!("go-judge: {e:#}")),
        };
        self.delete_files(cached).await;
        out
    }
}

fn ms_to_ns(ms: u64) -> u64 {
    ms * 1_000_000
}

fn kb_to_bytes(kb: u64) -> u64 {
    kb * 1024
}

/// Trims compiler output and hides sandbox paths.
fn clean_message(s: &str) -> String {
    let s = s.replace("/w/", "");
    let s = s.trim();
    if s.chars().count() > MESSAGE_MAX_CHARS {
        let cut: String = s.chars().take(MESSAGE_MAX_CHARS).collect();
        format!("{cut}\n…")
    } else {
        s.to_string()
    }
}
