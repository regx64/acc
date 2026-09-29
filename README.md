# acc

BOJ식 채점과 solved.ac식 레벨·레이팅을 합친 한국어 공개 온라인 저지.

- 공식 문제 + 사용자 출제 (검수 후 공개)
- 레벨 30단계: 수 체계 6그룹 ℕ ℤ ℚ ℝ ℂ ℍ × 5단계
- 레이팅: 맞힌 문제 중 상위 100문제의 (레벨)² 합. 레이팅 ≥ 90·L²이면 티어 L
- 대회, 난이도 투표, 클래스, 스트릭은 없음

코드는 MIT, 문제 지문은 CC BY-SA 4.0.

## 구조

```
[브라우저] ── Cloudflare Pages (SvelteKit, web/)
                 │ HTTPS · Cloudflare 터널
                 ▼
[Oracle Cloud A1 VM, 4 OCPU / 24GB ARM]
 ├─ acc-api     Rust, axum      crates/acc-api
 ├─ PostgreSQL
 ├─ Redis       채점 큐
 ├─ acc-worker  Rust            crates/acc-worker
 └─ go-judge    샌드박스        deploy/go-judge
[Cloudflare R2] ── 테스트 데이터 · 백업
```

| 경로 | 내용 |
| --- | --- |
| `crates/acc-core` | 판정, 언어, 출력 비교, 레벨·레이팅 계산, 큐 형식 |
| `crates/acc-judge` | 채점기 추상화(`Judge`)와 go-judge 구현, `acc-judge` CLI |
| `crates/acc-worker` | 큐에서 제출을 꺼내 채점하고 결과·레이팅을 기록 |
| `crates/acc-api` | REST API `/api/v1`, OpenAPI는 `acc-api --openapi` |
| `migrations/` | Postgres 스키마 |
| `web/` | SvelteKit 프론트엔드 |
| `deploy/` | 이미지, 배포 compose, 백업, VM 설정 |
| `scripts/smoke.sh` | 전체 스택 종단 간 테스트 |

제출 흐름: API가 제출을 `PENDING`으로 저장하고 Redis 큐에 id를 넣은 뒤 바로 응답합니다. 워커가 id를 꺼내 `JUDGING`으로 바꾸고, 테스트 데이터를 R2에서 받아 로컬에 캐시한 뒤, 한 번 컴파일하고 케이스별로 실행하다 첫 실패에서 멈춥니다. 결과를 저장하고 사용자·문제 상태를 갱신하며, 맞았으면 레이팅을 다시 계산합니다.

## 개발 환경 (Windows + WSL2)

편집은 Windows에서, 실행은 전부 WSL2(Ubuntu 24.04)에서 합니다. 저장소는 WSL 파일시스템의 `~/acc`에 두세요(`/mnt/c`는 느립니다). VS Code는 WSL 확장으로 엽니다.

필요한 도구:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Node 22
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt install -y nodejs
# Docker Desktop의 WSL 통합을 켜거나 WSL 안에 Docker Engine 설치
```

실행:

```bash
cp .env.example .env
mkdir -p var/cache
docker compose -f compose.dev.yml up -d          # Postgres, Redis, go-judge, MinIO

set -a; . ./.env; set +a
cargo run -p acc-api                             # :8080, 마이그레이션 자동 실행
cargo run -p acc-worker                          # 다른 터미널

cd web && cp .env.example .env && npm install && npm run dev   # :5173
```

`regx64` 핸들로 가입하면 관리자가 됩니다(`ADMIN_HANDLES`). 인증 메일은 `RESEND_API_KEY`가 없으면 API 로그에 찍힙니다.

로컬은 x86, 서버는 ARM입니다. ARM 이미지는 CI에서만 만듭니다.

## 테스트

```bash
cargo test --workspace                           # 단위 테스트
# 채점 코어(M1): 5개 언어 판정 + 샌드박스 공격
GO_JUDGE_URL=http://127.0.0.1:5050 GO_JUDGE_PATH=/usr/local/rust-bin:/usr/local/bin:/usr/bin:/bin \
  cargo test -p acc-judge -- --test-threads=2
# 전체 스택 (빈 DB에서, API 로그 경로 필요)
API_LOG=api.log scripts/smoke.sh
cd web && npm run check
```

CLI로 채점:

```bash
cargo run -p acc-judge -- --lang cpp17 --code sol.cpp --tests ./tests --time 1000 --mem 256
```

API를 바꾸면 프론트 타입을 다시 만듭니다: `cd web && npm run gen:api`.

## 브랜치와 CI

- `main`은 항상 배포 가능. 기능 브랜치에서 PR을 엽니다.
- PR마다: Rust 빌드·테스트·clippy·fmt, 프론트 빌드·타입체크, API 타입 동기화 확인, go-judge 이미지로 채점 테스트와 smoke 테스트.
- `main`에 머지하면 ARM 이미지를 GHCR에 올리고 Cloudflare Pages와 VM에 배포합니다.

배포와 운영은 [docs/deploy.md](docs/deploy.md)를 보세요.
