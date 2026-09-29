//! Command-line judge for milestone M1.
//!
//! ```text
//! acc-judge --lang cpp17 --code sol.cpp --tests ./tests --time 1000 --mem 256
//! ```
//! `--tests` holds `1.in`/`1.out`, `2.in`/`2.out`, ...

use std::path::{Path, PathBuf};

use acc_core::Language;
use acc_judge::gojudge::{GoJudge, GoJudgeConfig};
use acc_judge::{Input, Judge, JudgeRequest, TestCase};
use anyhow::{bail, Context};
use clap::Parser;

#[derive(Parser)]
#[command(about = "Judge a source file against a directory of n.in / n.out files")]
struct Args {
    /// c, cpp17, python3, java, rust
    #[arg(long)]
    lang: String,
    #[arg(long)]
    code: PathBuf,
    #[arg(long)]
    tests: PathBuf,
    /// Time limit in ms
    #[arg(long, default_value_t = 1000)]
    time: u64,
    /// Memory limit in MB
    #[arg(long, default_value_t = 256)]
    mem: u64,
    /// Overrides GO_JUDGE_URL
    #[arg(long)]
    judge_url: Option<String>,
    /// Print the failing case index and compile output as JSON
    #[arg(long)]
    json: bool,
}

pub fn load_cases(dir: &Path) -> anyhow::Result<Vec<TestCase>> {
    let mut idx: Vec<u32> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            name.strip_suffix(".in")?.parse().ok()
        })
        .collect();
    idx.sort_unstable();
    if idx.is_empty() {
        bail!("no n.in files in {}", dir.display());
    }
    idx.into_iter()
        .map(|i| {
            let input = std::fs::read(dir.join(format!("{i}.in")))?;
            let expected = std::fs::read(dir.join(format!("{i}.out")))
                .with_context(|| format!("missing {i}.out"))?;
            Ok(TestCase {
                input: Input::Bytes(input),
                expected,
            })
        })
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let language: Language = args.lang.parse().map_err(anyhow::Error::msg)?;
    let code = std::fs::read_to_string(&args.code)
        .with_context(|| format!("reading {}", args.code.display()))?;
    let cases = load_cases(&args.tests)?;

    let mut cfg = GoJudgeConfig::from_env();
    if let Some(u) = args.judge_url {
        cfg.url = u;
    }
    let judge = GoJudge::new(cfg);
    let out = judge
        .judge(&JudgeRequest {
            language,
            code,
            time_limit_ms: args.time,
            memory_limit_kb: args.mem * 1024,
            cases,
        })
        .await;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{} {}ms {}KB", out.status, out.time_ms, out.memory_kb);
        if let Some(m) = &out.message {
            eprintln!("{m}");
        }
    }
    Ok(())
}
