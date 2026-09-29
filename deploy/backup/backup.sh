#!/bin/bash
# Dumps the whole database to object storage and prunes dumps older than
# BACKUP_RETENTION_DAYS (default 30).
#
# Env: PGHOST PGUSER PGPASSWORD PGDATABASE, S3_ENDPOINT S3_ACCESS_KEY
#      S3_SECRET_KEY S3_BUCKET, optional ALERT_WEBHOOK_URL
set -euo pipefail

alert() {
  [[ -n "${ALERT_WEBHOOK_URL:-}" ]] || return 0
  curl -sS -m 10 -H 'content-type: application/json' \
    -d "{\"content\":\"[acc-backup] $1\"}" "$ALERT_WEBHOOK_URL" > /dev/null || true
}
trap 'alert "백업 실패 (line $LINENO)"' ERR

mc alias set store "$S3_ENDPOINT" "$S3_ACCESS_KEY" "$S3_SECRET_KEY" > /dev/null
name="backups/postgres/acc-$(date -u +%Y%m%dT%H%M%SZ).dump"
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
pg_dump --format=custom --compress=6 --no-owner --file="$tmp"
mc cp --quiet "$tmp" "store/$S3_BUCKET/$name" > /dev/null
echo "backup: uploaded $name ($(stat -c %s "$tmp") bytes)"
mc rm --quiet --recursive --force --older-than "${BACKUP_RETENTION_DAYS:-30}d" "store/$S3_BUCKET/backups/postgres/" > /dev/null || true
