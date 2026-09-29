//! Per-language compile and run commands.

use acc_core::Language;

pub struct Recipe {
    /// Compile command, or None when there is nothing to compile.
    pub compile: Option<Vec<&'static str>>,
    /// Files produced by compilation that the run step needs.
    pub artifacts: &'static [&'static str],
    pub run: Vec<String>,
    /// Max processes/threads while running.
    pub proc_limit: u64,
    /// Memory the sandbox allows on top of the problem limit, in KB.
    pub memory_overhead_kb: u64,
    /// Whether the run step needs the source file (interpreted languages).
    pub run_needs_source: bool,
}

pub const COMPILE_TIME_MS: u64 = 10_000;
pub const COMPILE_MEMORY_KB: u64 = 1024 * 1024;
pub const COMPILE_PROC_LIMIT: u64 = 128;

pub fn recipe(lang: Language, memory_limit_kb: u64) -> Recipe {
    match lang {
        Language::C => Recipe {
            compile: Some(vec!["gcc", "main.c", "-o", "main", "-O2", "-std=gnu11", "-Wall", "-lm", "-DONLINE_JUDGE"]),
            artifacts: &["main"],
            run: vec!["./main".into()],
            proc_limit: 16,
            memory_overhead_kb: 0,
            run_needs_source: false,
        },
        Language::Cpp17 => Recipe {
            compile: Some(vec!["g++", "main.cpp", "-o", "main", "-O2", "-std=gnu++17", "-Wall", "-lm", "-DONLINE_JUDGE"]),
            artifacts: &["main"],
            run: vec!["./main".into()],
            proc_limit: 16,
            memory_overhead_kb: 0,
            run_needs_source: false,
        },
        Language::Rust => Recipe {
            compile: Some(vec!["rustc", "--edition", "2021", "-O", "-o", "main", "main.rs", "--cfg", "online_judge"]),
            artifacts: &["main"],
            run: vec!["./main".into()],
            proc_limit: 16,
            memory_overhead_kb: 0,
            run_needs_source: false,
        },
        Language::Python3 => Recipe {
            compile: Some(vec![
                "python3",
                "-c",
                "import py_compile,sys\ntry:\n py_compile.compile('main.py',doraise=True)\nexcept py_compile.PyCompileError as e:\n print(e.msg,file=sys.stderr);sys.exit(1)",
            ]),
            artifacts: &[],
            run: vec!["python3".into(), "-S".into(), "main.py".into()],
            proc_limit: 16,
            memory_overhead_kb: 0,
            run_needs_source: true,
        },
        Language::Java => {
            let xmx = (memory_limit_kb / 1024).max(16);
            Recipe {
                compile: Some(vec![
                    "/bin/sh",
                    "-c",
                    "javac -J-Xms64m -J-Xmx512m -encoding UTF-8 -d out Main.java && jar -J-Djava.io.tmpdir=/w cfe main.jar Main -C out .",
                ]),
                artifacts: &["main.jar"],
                run: vec![
                    "java".into(),
                    format!("-Xmx{xmx}m"),
                    "-Xss64m".into(),
                    "-XX:+UseSerialGC".into(),
                    "-XX:TieredStopAtLevel=4".into(),
                    "-Dfile.encoding=UTF-8".into(),
                    "-DONLINE_JUDGE=1".into(),
                    "-cp".into(),
                    "main.jar".into(),
                    "Main".into(),
                ],
                proc_limit: 128,
                // JVM metaspace, code cache and thread stacks live outside the heap.
                memory_overhead_kb: 96 * 1024,
                run_needs_source: false,
            }
        }
    }
}
