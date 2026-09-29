#!/bin/bash
# Monthly restore test: restores the newest dump into a scratch database,
# checks that core tables came back, then drops it. Alerts on failure.
set -euo pipefail

alert() {
  [[ -n "${ALERT_WEBHOOK_URL:-}" ]] || return 0
  curl -sS -m 10 -H 'content-type: application/json' \
    -d "{\"content\":\"[acc-backup] $1\"}" "$ALERT_WEBHOOK_URL" > /dev/null || true
}
trap 'alert "복구 테스트 실패 (line $LINENO)"' ERR

SCRATCH=acc_restore_test
mc alias set store "$S3_ENDPOINT" "$S3_ACCESS_KEY" "$S3_SECRET_KEY" > /dev/null
latest=$(mc ls --json "store/$S3_BUCKET/backups/postgres/" | sed -n 's/.*"key":"\([^"]*\)".*/\1/p' | sort | tail -1)
[[ -n "$latest" ]] || { alert "복구 테스트: 백업이 없습니다"; exit 1; }
tmp=$(mktemp)
trap 'rm -f "$tmp"; dropdb --if-exists "$SCRATCH" || true' EXIT
mc cp --quiet "store/$S3_BUCKET/backups/postgres/$latest" "$tmp" > /dev/null

dropdb --if-exists "$SCRATCH"
createdb "$SCRATCH"
pg_restore --no-owner --dbname="$SCRATCH" "$tmp"
counts=$(psql -d "$SCRATCH" -At -c "SELECT (SELECT count(*) FROM users) || ' users, ' || (SELECT count(*) FROM problems) || ' problems, ' || (SELECT count(*) FROM submissions) || ' submissions'")
live=$(psql -At -c "SELECT count(*) FROM problems")
restored=$(psql -d "$SCRATCH" -At -c "SELECT count(*) FROM problems")
# Problems are rarely deleted, so the restored count must not exceed the live one.
if (( restored > live )); then
  alert "복구 테스트: 복원한 문제 수($restored)가 운영 DB($live)보다 많습니다"
  exit 1
fi
echo "restore-test: $latest restored OK ($counts)"
alert "복구 테스트 성공: $latest ($counts)"
