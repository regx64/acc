#!/usr/bin/env bash
# Judge throughput test for choosing QUEUE_LIMIT (M7).
#
# Inserts N submissions for one problem straight into the database, queues
# them all at once and measures how long the workers take to drain the
# queue. Run it against a staging copy, never the live database: it writes
# submissions under the given user.
#
#   DATABASE_URL=... REDIS_URL=... scripts/loadtest.sh <problem_id> <user_id> [N] [language] [code-file]
#
# QUEUE_LIMIT ≈ throughput × the longest wait you accept (e.g. 120 s).
set -euo pipefail

PROBLEM=${1:?problem id}
USER_ID=${2:?user id to submit as}
N=${3:-500}
LANG=${4:-python3}
CODE_FILE=${5:-}
: "${DATABASE_URL:?}" "${REDIS_URL:?}"

if [[ -n "$CODE_FILE" ]]; then
  CODE=$(cat "$CODE_FILE")
else
  CODE=$'a, b = map(int, input().split())\nprint(a + b)'
fi

echo "inserting $N submissions for problem $PROBLEM ($LANG)"
ids=$(psql "$DATABASE_URL" -qAt -v code="$CODE" -v lang="$LANG" -v uid="$USER_ID" -v pid="$PROBLEM" -v n="$N" <<'SQL'
INSERT INTO submissions (user_id, problem_id, language, code, code_length)
SELECT :uid, :pid, :'lang', :'code', length(:'code') FROM generate_series(1, :n)
RETURNING id;
SQL
)
first=$(head -1 <<< "$ids"); last=$(tail -1 <<< "$ids")

start=$(date +%s.%N)
printf 'LPUSH acc:queue s:%s\n' $ids | redis-cli -u "$REDIS_URL" > /dev/null

while true; do
  left=$(psql "$DATABASE_URL" -At -c "SELECT count(*) FROM submissions WHERE id BETWEEN $first AND $last AND status IN ('PENDING','JUDGING')")
  [[ "$left" == 0 ]] && break
  sleep 1
done
end=$(date +%s.%N)

psql "$DATABASE_URL" -At -F ' ' -c "SELECT status, count(*) FROM submissions WHERE id BETWEEN $first AND $last GROUP BY status"
awk -v s="$start" -v e="$end" -v n="$N" 'BEGIN {
  t = e - s; r = n / t
  printf "judged %d submissions in %.1f s: %.2f per second\n", n, t, r
  printf "QUEUE_LIMIT for a 60 s wait: %d, for a 120 s wait: %d\n", r * 60, r * 120
}'
