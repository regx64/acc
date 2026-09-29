//! Judging: compile once, run each test case, stop at the first failure.
//!
//! [`Judge`] is the abstraction the worker talks to, so the sandbox can be
//! swapped. [`gojudge::GoJudge`] is the current implementation.

pub mod gojudge;
pub mod lang;

use acc_core::{Language, Status};
use async_trait::async_trait;
use serde::Serialize;

/// Where a test input comes from.
#[derive(Debug, Clone)]
pub enum Input {
    /// Absolute path readable by the sandbox server.
    Path(String),
    /// Inline bytes.
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub input: Input,
    pub expected: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct JudgeRequest {
    pub language: Language,
    pub code: String,
    pub time_limit_ms: u64,
    pub memory_limit_kb: u64,
    pub cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct JudgeOutcome {
    pub status: Status,
    /// Max CPU time over the cases that ran, in ms.
    pub time_ms: u32,
    /// Max memory over the cases that ran, in KB.
    pub memory_kb: u32,
    /// Compile output for CE, internal detail for SE.
    pub message: Option<String>,
    /// Number of cases that passed.
    pub passed: usize,
    /// 1-based index of the failing case. Never shown to users.
    pub failed_case: Option<usize>,
}

impl JudgeOutcome {
    pub fn system_error(message: impl Into<String>) -> Self {
        JudgeOutcome {
            status: Status::Se,
            time_ms: 0,
            memory_kb: 0,
            message: Some(message.into()),
            passed: 0,
            failed_case: None,
        }
    }
}

#[async_trait]
pub trait Judge: Send + Sync {
    /// Judges one submission. Sandbox failures come back as `Status::Se`.
    async fn judge(&self, req: &JudgeRequest) -> JudgeOutcome;
}
