//! Problem and policy limits from the plan.

pub const TIME_LIMIT_MS: std::ops::RangeInclusive<u32> = 500..=10_000;
pub const MEMORY_LIMIT_KB: std::ops::RangeInclusive<u32> = 32 * 1024..=1024 * 1024;
pub const MAX_TESTCASES: usize = 100;
pub const MAX_TESTCASE_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_CODE_BYTES: usize = 64 * 1024;
pub const FIRST_PROBLEM_ID: i32 = 1000;
pub const MIN_PASSWORD_LEN: usize = 8;
/// Reports on a post or comment before it is hidden automatically.
pub const REPORT_HIDE_THRESHOLD: i64 = 3;
