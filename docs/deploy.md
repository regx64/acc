# 배포와 운영

## 한 번만 하는 준비

### Oracle Cloud VM

1. Always Free A1 인스턴스(4 OCPU / 24GB, Ubuntu 24.04 arm64)를 만듭니다.
2. VCN 보안 목록에서 22/tcp만 엽니다. 웹 트래픽은 Cloudflare 터널로 들어오므로 80/443을 열지 않습니다.
3. `sudo bash deploy/setup-vm.sh` — Docker, 방화벽, SSH 키 전용 로그인, fail2ban, 자동 보안 업데이트, `/opt/acc`.
4. `/opt/acc/.env`를 `deploy/.env.example`에서 만들고 `chmod 600`.

### Cloudflare

1. **R2**: 버킷 `acc-data`(테스트 데이터)와 `acc-backups`(DB 백업)를 만들고, 두 버킷에 쓸 수 있는 API 토큰을 발급해 `.env`에 넣습니다.
2. **터널**: Zero Trust → Networks → Tunnels에서 터널을 만들고 토큰을 `CLOUDFLARE_TUNNEL_TOKEN`에 넣습니다. Public hostname `api.<도메인>` → `http://api:8080`.
3. **Pages**: 프로젝트를 만들고(이름은 저장소 변수 `PAGES_PROJECT`), 환경 변수 `API_URL=https://api.<도메인>`, 비밀 `INTERNAL_TOKEN`(VM `.env`와 같은 값)을 넣습니다. 커스텀 도메인 `<도메인>`을 연결합니다.
4. `.env`의 `TRUST_CLOUDFLARE=1`: API가 `CF-Connecting-IP`를 믿습니다.

### Resend

도메인을 인증하고 API 키를 `RESEND_API_KEY`, 보내는 주소를 `MAIL_FROM`에 넣습니다.

### GitHub

- Secrets: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`, `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`, `DEPLOY_KNOWN_HOSTS`
- Variables: `PAGES_PROJECT`
- VM에서 GHCR 이미지를 받을 수 있게 패키지를 공개로 두거나 `docker login ghcr.io`를 해 둡니다.

## 배포

`main`에 머지하면 `.github/workflows/deploy.yml`이 돌아갑니다.

1. ARM 러너에서 `acc`, `acc-go-judge`, `acc-backup` 이미지를 빌드해 GHCR에 올립니다(태그: 커밋 SHA).
2. 프론트를 빌드해 Cloudflare Pages에 올립니다.
3. VM에 `compose.yml`을 복사하고 `ACC_TAG`를 바꾼 뒤 `docker compose pull && up -d`.
   워커는 SIGTERM을 받으면 진행 중인 채점을 마치고 종료합니다(`stop_grace_period: 5m`).
4. API 헬스체크가 통과할 때까지 기다립니다.

되돌리기: VM에서 `.env`의 `ACC_TAG`를 이전 SHA로 바꾸고 `docker compose up -d`.

## 백업과 복구

`backup` 컨테이너가 다음을 합니다(UTC 기준 cron).

| 언제 | 무엇 |
| --- | --- |
| 매일 03:30 KST | `pg_dump` 전체 → R2 `acc-backups/backups/postgres/`, 30일 지난 덤프 삭제 |
| 매월 1일 04:00 KST | 최신 덤프를 임시 DB에 복원해 확인 후 삭제, 결과를 웹훅으로 알림 |

지금 바로: `docker compose run --rm backup now`, `docker compose run --rm backup restore-test`.

실제 복구:

```bash
mc cp store/acc-backups/backups/postgres/acc-<시각>.dump ./restore.dump
docker compose stop api worker
docker compose exec -T postgres dropdb -U acc acc
docker compose exec -T postgres createdb -U acc acc
docker compose exec -T postgres pg_restore -U acc -d acc --no-owner < restore.dump
docker compose start api worker
```

## 모니터링과 알림

- **외부 핑**: UptimeRobot 등으로 `https://api.<도메인>/api/v1/health`와 `https://<도메인>/`을 5분마다 확인하고, 다운 알림을 디스코드로 보냅니다.
- **관리 페이지** `/admin`: 채점 대기열 길이, 24시간 채점 오류율, 검수·신고·오타 제보 대기.
- **웹훅** (`ALERT_WEBHOOK_URL`): 채점 오류(SE), 오타 제보, 검수 요청, 백업·복구 테스트 결과.
- **점검 공지**: `/admin`에서 배너 문구를 넣으면 모든 페이지 위에 뜹니다.

## 언어별 시간 보정 (M6에서 확정)

`crates/acc-core/src/language.rs`의 `time_factor`는 임시 값입니다(C/C++/Rust ×1, Java ×2+1초, Python ×3+2초). 운영 서버에서 같은 알고리즘을 언어별로 재어 확정합니다.

```bash
# VM에서: 각 언어 풀이로 같은 입력을 여러 번 채점해 시간을 비교
cargo run -p acc-judge -- --lang python3 --code sol.py --tests ./bench --time 10000 --json
```

## 공개 전 점검 (M7)

- 운영 go-judge에 `cargo test -p acc-judge` 공격 테스트를 다시 돌립니다(`GO_JUDGE_URL`을 VM 내부 주소로).
- 부하 테스트로 `QUEUE_LIMIT`을 정합니다: 대기열이 이 길이를 넘으면 제출을 잠시 거절합니다.
- 약관 3종(`web/src/lib/legal/*.md`)의 `[…]` 자리를 채우고 게시합니다.
- 공식 문제 50개(`problems/`)를 올리고 레벨을 확정합니다.
