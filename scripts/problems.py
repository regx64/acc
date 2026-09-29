#!/usr/bin/env python3
"""Build and upload official problems from problems/<dir>/.

A problem directory holds:

  problem.toml   title, time_limit_ms, memory_limit_mb, level, solution
  statement.md   "# 문제", "# 입력", "# 출력" and optional "# 힌트" sections
  gen.py         SAMPLES: list[str] and tests(rng) -> list[str]
  sol.py|sol.cpp reference solution (reads stdin, writes stdout)
  brute.py       optional slow solution, cross-checked on tests whose input
                 is at most BRUTE_MAX_INPUT bytes (gen.py may override it);
                 it exits with status 3 to skip an input too big for it

Usage:
  scripts/problems.py build [dir ...]
  scripts/problems.py upload --api URL --login HANDLE --password PW [--publish] [dir ...]

`build` runs the generator and the reference solution and writes
build/problems/<dir>/ (tests.zip plus sample outputs). `upload` creates or
updates each problem through the API (matching on title), uploads the test
zip, runs the reference-solution check and, with --publish, publishes it at
the level from problem.toml.
"""

from __future__ import annotations

import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
import random
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import urllib.error
import urllib.request
import uuid
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBLEMS = ROOT / "problems"
BUILD = ROOT / "build" / "problems"
MAX_TESTS = 100
MAX_BYTES = 50 * 1024 * 1024
BRUTE_MAX_INPUT = 4000
# A brute solution exits with this status to skip an input it cannot handle.
SKIP_EXIT = 3


def fail(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def load_gen(pdir: Path):
    spec = importlib.util.spec_from_file_location(f"gen_{pdir.name}", pdir / "gen.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def parse_statement(text: str) -> dict:
    sections = {"문제": "legend", "입력": "input", "출력": "output", "힌트": "hint"}
    out: dict[str, str] = {}
    current = None
    buf: list[str] = []
    for line in text.splitlines():
        m = re.match(r"^# (\S+)\s*$", line)
        if m and m.group(1) in sections:
            if current:
                out[current] = "\n".join(buf).strip()
            current, buf = sections[m.group(1)], []
        else:
            buf.append(line)
    if current:
        out[current] = "\n".join(buf).strip()
    for key in ("legend", "input", "output"):
        if not out.get(key):
            raise ValueError(f"statement.md is missing a section for {key}")
    return out


class Runner:
    """Runs a solution file on inputs, compiling C++ once."""

    def __init__(self, path: Path, workdir: Path):
        self.path = path
        if path.suffix == ".cpp":
            exe = workdir / (path.stem + ".bin")
            r = subprocess.run(["g++", "-O2", "-std=gnu++17", "-w", "-o", str(exe), str(path)], capture_output=True)
            if r.returncode != 0:
                raise RuntimeError(f"compiling {path.name} failed:\n{r.stderr.decode()[:2000]}")
            self.cmd = [str(exe)]
        elif path.suffix == ".py":
            self.cmd = [sys.executable, str(path)]
        else:
            raise ValueError(f"unsupported solution: {path.name}")

    def run(self, data: str) -> str | None:
        r = subprocess.run(self.cmd, input=data.encode(), capture_output=True, timeout=60)
        if r.returncode == SKIP_EXIT:
            return None
        if r.returncode != 0:
            raise RuntimeError(f"{self.path.name} exited {r.returncode}: {r.stderr.decode()[:500]}")
        return r.stdout.decode()


def language_of(path: Path) -> str:
    return {"py": "python3", "cpp": "cpp17"}[path.suffix[1:]]


def normalize(s: str) -> str:
    lines = [l.rstrip() for l in s.replace("\r\n", "\n").split("\n")]
    while lines and not lines[-1]:
        lines.pop()
    return "\n".join(lines)


def build_one(pdir: Path) -> dict:
    meta = tomllib.loads((pdir / "problem.toml").read_text())
    statement = parse_statement((pdir / "statement.md").read_text())
    gen = load_gen(pdir)
    seed = int(hashlib.sha256(pdir.name.encode()).hexdigest()[:12], 16)
    rng = random.Random(seed)
    samples = list(gen.SAMPLES)
    tests = list(gen.tests(rng))
    if not samples:
        raise ValueError("no samples")
    if not 1 <= len(tests) <= MAX_TESTS:
        raise ValueError(f"{len(tests)} tests (1..{MAX_TESTS} allowed)")

    out_dir = BUILD / pdir.name
    shutil.rmtree(out_dir, ignore_errors=True)
    out_dir.mkdir(parents=True)
    with tempfile.TemporaryDirectory() as tmp:
        sol = Runner(pdir / meta["solution"], Path(tmp))
        brute = Runner(pdir / "brute.py", Path(tmp)) if (pdir / "brute.py").exists() else None
        sample_out = [sol.run(s) for s in samples]
        outputs = []
        checked = 0
        for i, t in enumerate(tests, 1):
            if not t.endswith("\n"):
                t += "\n"
                tests[i - 1] = t
            o = sol.run(t)
            outputs.append(o)
            if brute and len(t) <= getattr(gen, "BRUTE_MAX_INPUT", BRUTE_MAX_INPUT):
                b = brute.run(t)
                if b is None:
                    continue
                if normalize(b) != normalize(o):
                    (out_dir / "mismatch.in").write_text(t)
                    raise ValueError(f"test {i}: solution and brute differ (input saved to mismatch.in)")
                checked += 1
        for s, o in zip(samples, sample_out):
            b = brute.run(s) if brute else o
            if b is not None and normalize(b) != normalize(o):
                raise ValueError("sample: solution and brute differ")

    total = sum(len(t.encode()) + len(o.encode()) for t, o in zip(tests, outputs))
    if total > MAX_BYTES:
        raise ValueError(f"test data is {total} bytes (max {MAX_BYTES})")
    with zipfile.ZipFile(out_dir / "tests.zip", "w", zipfile.ZIP_DEFLATED) as z:
        for i, (t, o) in enumerate(zip(tests, outputs), 1):
            z.writestr(f"{i}.in", t)
            z.writestr(f"{i}.out", o)
    solution_path = pdir / meta["solution"]
    payload = {
        "title": meta["title"],
        "time_limit_ms": meta["time_limit_ms"],
        "memory_limit_mb": meta["memory_limit_mb"],
        "proposed_level": meta["level"],
        "statement": {
            "legend": statement["legend"],
            "input": statement["input"],
            "output": statement["output"],
            "hint": statement.get("hint"),
            "samples": [{"input": s, "output": o} for s, o in zip(samples, sample_out)],
        },
        "solution_language": language_of(solution_path),
        "solution_code": solution_path.read_text(),
        "agree_terms": True,
    }
    (out_dir / "problem.json").write_text(json.dumps(payload, ensure_ascii=False, indent=1))
    print(f"  {pdir.name}: {len(tests)} tests, {total / 1024:.0f} KB" + (f", brute-checked {checked}" if brute else ""))
    return payload


def problem_dirs(names: list[str]) -> list[Path]:
    if names:
        return [PROBLEMS / Path(n).name for n in names]
    return sorted(p for p in PROBLEMS.iterdir() if (p / "problem.toml").exists())


class Api:
    def __init__(self, base: str):
        self.base = base.rstrip("/")
        self.opener = urllib.request.build_opener(
            urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()),
            urllib.request.ProxyHandler({}),
        )

    def call(self, method: str, path: str, body=None, raw: bytes | None = None, ctype: str | None = None):
        data = raw if raw is not None else (json.dumps(body).encode() if body is not None else None)
        req = urllib.request.Request(self.base + path, data=data, method=method)
        if body is not None:
            req.add_header("content-type", "application/json")
        if ctype:
            req.add_header("content-type", ctype)
        try:
            with self.opener.open(req) as r:
                text = r.read().decode()
                return json.loads(text) if text else None
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"{method} {path}: {e.code} {e.read().decode()[:300]}") from None

    def upload_zip(self, path: str, zip_path: Path):
        boundary = uuid.uuid4().hex
        body = (
            f"--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"tests.zip\"\r\n"
            f"Content-Type: application/zip\r\n\r\n"
        ).encode() + zip_path.read_bytes() + f"\r\n--{boundary}--\r\n".encode()
        self.call("PUT", path, raw=body, ctype=f"multipart/form-data; boundary={boundary}")


def upload(args) -> None:
    api = Api(args.api)
    api.call("POST", "/auth/login", {"login": args.login, "password": args.password})
    mine = {p["title"]: p for p in api.call("GET", "/my/problems")}
    for pdir in problem_dirs(args.dirs):
        payload = json.loads((BUILD / pdir.name / "problem.json").read_text())
        level = payload["proposed_level"]
        existing = mine.get(payload["title"])
        if existing and existing["status"] == "PUBLIC":
            print(f"  {pdir.name}: already public as {existing['id']}, skipped")
            continue
        if existing:
            pid = existing["id"]
            api.call("PUT", f"/my/problems/{pid}", payload)
        else:
            pid = api.call("POST", "/my/problems", payload)["id"]
        api.upload_zip(f"/my/problems/{pid}/testcases", BUILD / pdir.name / "tests.zip")
        api.call("POST", f"/my/problems/{pid}/validate")
        for _ in range(240):
            p = api.call("GET", f"/my/problems/{pid}")
            if p["validation_status"] != "PENDING":
                break
            time.sleep(0.5)
        if p["validation_status"] != "PASSED":
            print(f"  {pdir.name} ({pid}): validation {p['validation_status']}: {p['validation_message']}")
            continue
        if args.publish:
            api.call("POST", f"/admin/problems/{pid}/decision", {"approve": True, "level": level})
        print(f"  {pdir.name}: problem {pid} {'published' if args.publish else 'validated'} ({p['validation_message']})")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("dirs", nargs="*")
    u = sub.add_parser("upload")
    u.add_argument("--api", required=True)
    u.add_argument("--login", required=True)
    u.add_argument("--password", required=True)
    u.add_argument("--publish", action="store_true")
    u.add_argument("dirs", nargs="*")
    args = ap.parse_args()
    if args.cmd == "build":
        dirs = problem_dirs(args.dirs)
        print(f"building {len(dirs)} problems")
        errors = 0
        for d in dirs:
            try:
                build_one(d)
            except Exception as e:  # report every broken package, not just the first
                errors += 1
                print(f"  {d.name}: FAILED: {e}")
        if errors:
            fail(f"{errors} problem(s) failed")
    else:
        upload(args)


if __name__ == "__main__":
    main()
