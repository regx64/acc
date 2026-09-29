//! M1 acceptance tests against a live go-judge.
//!
//! Run with `GO_JUDGE_URL=http://127.0.0.1:5050 cargo test -p acc-judge -- --test-threads=2`.
//! Skipped when `GO_JUDGE_URL` is unset.

use acc_core::{Language, Status};
use acc_judge::gojudge::{GoJudge, GoJudgeConfig};
use acc_judge::{Input, Judge, JudgeOutcome, JudgeRequest, TestCase};

fn judge() -> Option<GoJudge> {
    std::env::var("GO_JUDGE_URL").ok()?;
    Some(GoJudge::new(GoJudgeConfig::from_env()))
}

/// A+B with three cases.
fn cases() -> Vec<TestCase> {
    [
        ("1 2\n", "3\n"),
        ("-5 5\n", "0\n"),
        ("1000000 2000000\n", "3000000\n"),
    ]
    .into_iter()
    .map(|(i, o)| TestCase {
        input: Input::Bytes(i.into()),
        expected: o.into(),
    })
    .collect()
}

async fn run(j: &GoJudge, lang: Language, code: &str) -> JudgeOutcome {
    run_with(j, lang, code, 1000, 128 * 1024).await
}

async fn run_with(
    j: &GoJudge,
    lang: Language,
    code: &str,
    time_ms: u64,
    mem_kb: u64,
) -> JudgeOutcome {
    j.judge(&JudgeRequest {
        language: lang,
        code: code.into(),
        time_limit_ms: time_ms,
        memory_limit_kb: mem_kb,
        cases: cases(),
    })
    .await
}

struct Programs {
    lang: Language,
    ac: &'static str,
    wa: &'static str,
    tle: &'static str,
    mle: &'static str,
    re: &'static str,
    ce: &'static str,
}

const C: Programs = Programs {
    lang: Language::C,
    ac: "#include <stdio.h>\nint main(){long long a,b;scanf(\"%lld %lld\",&a,&b);printf(\"%lld\\n\",a+b);return 0;}",
    wa: "#include <stdio.h>\nint main(){long long a,b;scanf(\"%lld %lld\",&a,&b);printf(\"%lld\\n\",a-b);return 0;}",
    tle: "int main(){volatile unsigned long x=0;for(;;)x++;}",
    mle: "#include <stdlib.h>\n#include <string.h>\nint main(){for(;;){char*p=malloc(1<<24);if(!p)return 1;memset(p,1,1<<24);}}",
    re: "#include <stdio.h>\nint main(){int *p=0;*p=1;return 0;}",
    ce: "int main(){ return }",
};

const CPP: Programs = Programs {
    lang: Language::Cpp17,
    ac: "#include <bits/stdc++.h>\nint main(){long long a,b;std::cin>>a>>b;std::cout<<a+b<<'\\n';}",
    wa: "#include <bits/stdc++.h>\nint main(){long long a,b;std::cin>>a>>b;std::cout<<a*b<<'\\n';}",
    tle: "int main(){volatile unsigned long x=0;for(;;)x++;}",
    mle: "#include <vector>\n#include <cstring>\nint main(){std::vector<char*> v;for(;;){char*p=new char[1<<24];std::memset(p,1,1<<24);v.push_back(p);}}",
    re: "#include <stdexcept>\nint main(){throw std::runtime_error(\"x\");}",
    ce: "#include <bits/stdc++.h>\nint main(){ undefined_fn(); }",
};

const PY: Programs = Programs {
    lang: Language::Python3,
    ac: "a,b=map(int,input().split())\nprint(a+b)",
    wa: "a,b=map(int,input().split())\nprint(a+b+1)",
    tle: "while True:\n    pass",
    mle: "x=[]\nwhile True:\n    x.append(bytearray(1<<24))",
    re: "raise SystemExit(3)",
    ce: "def f(:\n  pass",
};

const JAVA: Programs = Programs {
    lang: Language::Java,
    ac: "import java.util.*;\npublic class Main{public static void main(String[] a){Scanner s=new Scanner(System.in);long x=s.nextLong(),y=s.nextLong();System.out.println(x+y);}}",
    wa: "import java.util.*;\npublic class Main{public static void main(String[] a){Scanner s=new Scanner(System.in);long x=s.nextLong(),y=s.nextLong();System.out.println(x);}}",
    tle: "public class Main{public static void main(String[] a){long x=0;while(true){x++;if(x==-1)break;}System.out.println(x);}}",
    mle: "import java.util.*;\npublic class Main{public static void main(String[] a){List<long[]> l=new ArrayList<>();while(true){l.add(new long[1<<20]);}}}",
    re: "public class Main{public static void main(String[] a){int[] x=new int[1];System.out.println(x[5]);}}",
    ce: "public class Main{public static void main(String[] a){ int x = \"s\"; }}",
};

const RUST: Programs = Programs {
    lang: Language::Rust,
    ac: "use std::io::*;\nfn main(){let mut s=String::new();stdin().read_to_string(&mut s).unwrap();let v:Vec<i64>=s.split_whitespace().map(|x|x.parse().unwrap()).collect();println!(\"{}\",v[0]+v[1]);}",
    wa: "fn main(){println!(\"42\");}",
    tle: "fn main(){let mut x:u64=0;loop{x=std::hint::black_box(x+1);}}",
    mle: "fn main(){let mut v:Vec<Vec<u8>>=Vec::new();loop{v.push(vec![1u8;1<<24]);}}",
    re: "fn main(){let v:Vec<i32>=Vec::new();let i=std::hint::black_box(3);println!(\"{}\",v[i]);}",
    ce: "fn main(){ let x: i32 = \"no\"; }",
};

async fn check_language(p: &Programs) {
    let Some(j) = judge() else { return };
    let cases: [(&str, &str, Status); 6] = [
        ("ac", p.ac, Status::Ac),
        ("wa", p.wa, Status::Wa),
        ("tle", p.tle, Status::Tle),
        ("mle", p.mle, Status::Mle),
        ("re", p.re, Status::Re),
        ("ce", p.ce, Status::Ce),
    ];
    for (name, code, want) in cases {
        let out = run(&j, p.lang, code).await;
        assert_eq!(out.status, want, "{} {name}: {out:?}", p.lang);
        if want == Status::Ce {
            assert!(
                out.message.as_deref().is_some_and(|m| !m.is_empty()),
                "CE needs a message"
            );
        }
    }
}

#[tokio::test]
async fn lang_c() {
    check_language(&C).await;
}

#[tokio::test]
async fn lang_cpp17() {
    check_language(&CPP).await;
}

#[tokio::test]
async fn lang_python3() {
    check_language(&PY).await;
}

#[tokio::test]
async fn lang_java() {
    check_language(&JAVA).await;
}

#[tokio::test]
async fn lang_rust() {
    check_language(&RUST).await;
}

#[tokio::test]
async fn stops_at_first_failure() {
    let Some(j) = judge() else { return };
    // Correct only for the first case.
    let out = run(&j, Language::Python3, "input()\nprint(3)").await;
    assert_eq!(out.status, Status::Wa);
    assert_eq!(out.passed, 1);
    assert_eq!(out.failed_case, Some(2));
}

#[tokio::test]
async fn whitespace_rules() {
    let Some(j) = judge() else { return };
    let out = run(
        &j,
        Language::Python3,
        "a,b=map(int,input().split())\nprint(a+b, end='   \\n\\n\\n')",
    )
    .await;
    assert_eq!(out.status, Status::Ac);
    let out = run(
        &j,
        Language::Python3,
        "a,b=map(int,input().split())\nprint(' '+str(a+b))",
    )
    .await;
    assert_eq!(out.status, Status::Wa);
}

/// Hostile programs must get a verdict and leave the sandbox usable.
#[tokio::test]
async fn attacks() {
    let Some(j) = judge() else { return };
    let attacks: [(&str, Language, &str, &[Status]); 9] = [
        (
            "fork bomb",
            Language::C,
            "#include <unistd.h>\nint main(){for(;;)fork();}",
            &[Status::Tle, Status::Re, Status::Mle],
        ),
        (
            "python fork bomb",
            Language::Python3,
            "import os\nwhile True:\n    try: os.fork()\n    except OSError: pass",
            &[Status::Tle, Status::Re, Status::Mle],
        ),
        (
            "huge output",
            Language::Cpp17,
            "#include <cstdio>\nint main(){for(;;)fputs(\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\n\",stdout);}",
            &[Status::Wa, Status::Tle, Status::Mle],
        ),
        (
            "sleep forever",
            Language::Python3,
            "import time\ntime.sleep(1000)",
            &[Status::Tle],
        ),
        (
            "read host files",
            Language::Python3,
            "print(open('/etc/shadow').read())",
            &[Status::Re],
        ),
        (
            "write outside workdir",
            Language::Python3,
            "open('/usr/pwned','w').write('x')",
            &[Status::Re],
        ),
        (
            "network",
            Language::Python3,
            "import socket\ns=socket.create_connection(('1.1.1.1',80),timeout=2)\nprint('connected')",
            &[Status::Re],
        ),
        (
            "fill tmpfs",
            Language::C,
            "#include <stdio.h>\nint main(){FILE*f=fopen(\"big\",\"w\");static char b[1<<20];for(int i=0;i<4096;i++)fwrite(b,1,sizeof b,f);fclose(f);return 1;}",
            &[Status::Re, Status::Tle, Status::Mle],
        ),
        (
            "stack overflow",
            Language::C,
            "int f(int x){volatile char b[4096];b[0]=x;return f(x+1)+b[0];}\nint main(){return f(0);}",
            &[Status::Re, Status::Mle],
        ),
    ];
    for (name, lang, code, allowed) in attacks {
        let out = run_with(&j, lang, code, 1000, 64 * 1024).await;
        assert!(allowed.contains(&out.status), "{name}: got {out:?}");
        // The sandbox still works after each attack.
        let ok = run(&j, Language::C, C.ac).await;
        assert_eq!(ok.status, Status::Ac, "after {name}: {ok:?}");
    }
}
