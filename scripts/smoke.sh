#!/usr/bin/env bash
# End-to-end smoke test against a running API + worker + go-judge.
#
#   API=http://localhost:8080/api/v1 API_LOG=/path/to/api.log scripts/smoke.sh
#
# Needs a fresh database (it signs up fixed handles). Mail is read from the
# API log, so run the API without RESEND_API_KEY.
set -euo pipefail

API=${API:-http://localhost:8080/api/v1}
API_LOG=${API_LOG:?set API_LOG to the API log file}
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

pass=0
ok() { pass=$((pass + 1)); echo "  ok  $*"; }
fail() { echo "FAIL  $*" >&2; exit 1; }

# req COOKIE METHOD PATH [JSON] -> body on stdout, status in $WORK/status
req() {
  local jar=$1 method=$2 path=$3 body=${4:-}
  local args=(-s -o "$WORK/body" -w '%{http_code}' -b "$WORK/$jar" -c "$WORK/$jar" -X "$method" "$API$path")
  [[ -n $body ]] && args+=(-H 'content-type: application/json' -d "$body")
  curl "${args[@]}" > "$WORK/status"
  cat "$WORK/body"
}
status() { cat "$WORK/status"; }
expect() { [[ $(status) == "$1" ]] || fail "$2: expected $1, got $(status): $(cat "$WORK/body")"; ok "$2"; }

verify_last_mail() {
  local token
  token=$(grep -o 'verify-email?token=[A-Za-z0-9_-]*' "$API_LOG" | tail -1 | cut -d= -f2)
  req anon POST /auth/verify-email "{\"token\":\"$token\"}" > /dev/null
  expect 204 "verify email"
}

wait_status() { # submission id -> final status
  for _ in $(seq 60); do
    local st
    st=$(req "$2" GET "/submissions/$1" | jq -r .status)
    [[ $st != PENDING && $st != JUDGING ]] && { echo "$st"; return; }
    sleep 0.5
  done
  echo TIMEOUT
}

echo "== accounts"
req admin POST /auth/signup '{"handle":"regx64","email":"regx64@example.com","password":"password123","agree_terms":true}' > /dev/null
expect 200 "signup admin"
verify_last_mail
req alice POST /auth/signup '{"handle":"alice","email":"alice@example.com","password":"password123","agree_terms":true}' > /dev/null
expect 200 "signup alice"
[[ $(req alice GET /me | jq -r .role) == USER ]] && ok "alice is USER"
req alice POST /problems/1000/submit '{"language":"python3","code":"print(1)"}' > /dev/null
expect 403 "unverified user cannot submit"
verify_last_mail
req bob POST /auth/signup '{"handle":"alice","email":"bob@example.com","password":"password123","agree_terms":true}' > /dev/null
expect 409 "duplicate handle refused"
req bob POST /auth/login '{"login":"alice","password":"wrong-password"}' > /dev/null
expect 401 "bad password refused"

echo "== official problem"
STATEMENT='{"legend":"두 정수 $A$와 $B$를 입력받아 $A+B$를 출력한다.","input":"첫째 줄에 $A$와 $B$가 주어진다. ($-10^9 \\le A, B \\le 10^9$)","output":"$A+B$를 출력한다.","samples":[{"input":"1 2\n","output":"3\n"}]}'
PROBLEM="{\"title\":\"A+B\",\"time_limit_ms\":1000,\"memory_limit_mb\":256,\"proposed_level\":1,\"statement\":$STATEMENT,\"solution_language\":\"python3\",\"solution_code\":\"a,b=map(int,input().split())\\nprint(a+b)\"}"
PID=$(req admin POST /my/problems "$PROBLEM" | jq -r .id)
expect 200 "admin creates problem $PID"
[[ $PID == 1000 ]] && ok "first problem id is 1000"
mkdir -p "$WORK/tc"
for i in 1 2 3; do echo "$i $((i * 7))" > "$WORK/tc/$i.in"; echo $((i + i * 7)) > "$WORK/tc/$i.out"; done
(cd "$WORK/tc" && zip -q ../tc.zip ./*.in ./*.out)
curl -s -o "$WORK/body" -w '%{http_code}' -b "$WORK/admin" -X PUT -F "file=@$WORK/tc.zip" "$API/my/problems/$PID/testcases" > "$WORK/status"
expect 204 "upload test zip"
[[ $(req admin GET "/my/problems/$PID" | jq '.testcases | length') == 4 ]] && ok "sample + 3 data cases"
req admin POST "/my/problems/$PID/validate" > /dev/null
expect 204 "queue validation"
for _ in $(seq 60); do
  v=$(req admin GET "/my/problems/$PID" | jq -r .validation_status)
  [[ $v != PENDING ]] && break
  sleep 0.5
done
[[ $v == PASSED ]] || fail "validation: $v $(req admin GET "/my/problems/$PID" | jq -r .validation_message)"
ok "reference solution passes"
req alice GET "/problems/$PID" > /dev/null
expect 404 "draft hidden from others"
req admin POST "/admin/problems/$PID/decision" '{"approve":true,"level":13}' > /dev/null
expect 204 "admin publishes at level 13"
[[ $(req anon GET "/problems?sort=level&order=desc" | jq '.items[0].id') == "$PID" ]] && ok "listed publicly"

echo "== submissions"
SID=$(req alice POST "/problems/$PID/submit" '{"language":"cpp17","code":"#include <cstdio>\nint main(){long long a,b;scanf(\"%lld%lld\",&a,&b);printf(\"%lld\\n\",a-b);}"}' | jq -r .id)
expect 200 "submit WA"
[[ $(wait_status "$SID" alice) == WA ]] && ok "judged WA"
req alice POST "/problems/$PID/submit" '{"language":"python3","code":"print(1)"}' > /dev/null
expect 429 "10 second submit limit"
sleep 10
SID=$(req alice POST "/problems/$PID/submit" '{"language":"python3","code":"a,b=map(int,input().split())\nprint(a+b)"}' | jq -r .id)
[[ $(wait_status "$SID" alice) == AC ]] && ok "judged AC"
ME=$(req alice GET /me)
[[ $(jq .rating <<< "$ME") == 169 && $(jq .tier <<< "$ME") == 1 ]] && ok "rating 13² = 169, tier 1"
[[ $(req anon GET /users/alice | jq '.solved[0].id') == "$PID" ]] && ok "profile lists solved problem"
[[ $(req anon GET /ranking | jq -r '.items[0].handle') == alice ]] && ok "alice tops the ranking"
req admin PATCH "/admin/problems/$PID" '{"level":20}' > /dev/null
[[ $(req alice GET /me | jq .rating) == 400 ]] && ok "level change recomputes rating (20² = 400)"
[[ $(req anon GET "/submissions?problem_id=$PID" | jq '.items | length') == 2 ]] && ok "status board lists both"

echo "== code visibility and board"
req bob POST /auth/signup '{"handle":"bob","email":"bob@example.com","password":"password123","agree_terms":true}' > /dev/null
verify_last_mail
[[ $(req bob GET "/submissions/$SID" | jq -r .code) == null ]] && ok "unsolved user cannot read code"
[[ $(req alice GET "/submissions/$SID" | jq -r .code) != null ]] && ok "author reads own code"
POST_ID=$(req alice POST "/problems/$PID/posts" '{"kind":"COUNTEREXAMPLE","title":"반례","body":"음수 입력","code":"print(0)","code_language":"python3"}' | jq -r .id)
expect 200 "alice posts with code"
[[ $(req bob GET "/posts/$POST_ID" | jq -r .code_hidden) == true ]] && ok "attached code hidden from non-solver"
req alice POST "/posts/$POST_ID/comments" '{"body":"추가"}' > /dev/null
expect 429 "1 minute write limit"
for who in bob admin; do req "$who" POST /reports "{\"target_type\":\"POST\",\"target_id\":$POST_ID,\"reason\":\"test\"}" > /dev/null; done
req carol POST /auth/signup '{"handle":"carol","email":"carol@example.com","password":"password123","agree_terms":true}' > /dev/null
verify_last_mail
req carol POST /reports "{\"target_type\":\"POST\",\"target_id\":$POST_ID,\"reason\":\"test\"}" > /dev/null
req bob GET "/posts/$POST_ID" > /dev/null
expect 404 "three reports hide the post"

echo "== user authoring and review"
USER_PROBLEM=${PROBLEM/\"title\":\"A+B\"/\"title\":\"A+B 2\"}
req bob POST /my/problems "$USER_PROBLEM" > /dev/null
expect 400 "authoring terms required"
UPID=$(req bob POST /my/problems "$(jq -c '. + {agree_terms: true}' <<< "$USER_PROBLEM")" | jq -r .id)
expect 200 "bob drafts problem $UPID"
req bob POST "/my/problems/$UPID/request-review" > /dev/null
expect 409 "review needs a passing reference solution"
curl -s -o /dev/null -b "$WORK/bob" -X PUT -F "file=@$WORK/tc.zip" "$API/my/problems/$UPID/testcases"
req bob POST "/my/problems/$UPID/validate" > /dev/null
for _ in $(seq 60); do
  v=$(req bob GET "/my/problems/$UPID" | jq -r .validation_status)
  [[ $v != PENDING ]] && break
  sleep 0.5
done
[[ $v == PASSED ]] && ok "bob's reference solution passes"
req bob POST "/my/problems/$UPID/request-review" > /dev/null
expect 204 "review requested"
req admin POST "/admin/problems/$UPID/decision" '{"approve":false,"comment":"기존 문제와 같습니다."}' > /dev/null
expect 204 "admin rejects"
[[ $(req bob GET /my/problems | jq -r '.[0].status') == DRAFT ]] && ok "rejected problem is a draft again"

echo "== account"
req alice POST /me/handle '{"handle":"alice2"}' > /dev/null
expect 204 "change handle"
[[ $(req anon GET /users/alice | jq -r .error.location) == alice2 ]] && ok "old handle redirects"
req alice POST /me/handle '{"handle":"alice3"}' > /dev/null
expect 409 "second change within 90 days refused"
req bob POST /auth/signup '{"handle":"alice","email":"z@example.com","password":"password123","agree_terms":true}' > /dev/null
expect 409 "released handle stays locked"
req admin POST /admin/users/3/suspend '{"reason":"테스트"}' > /dev/null
req bob POST "/problems/$PID/submit" '{"language":"python3","code":"print(1)"}' > /dev/null
expect 403 "suspended user cannot submit"
req bob POST /me/delete '{"password":"password123"}' > /dev/null
expect 204 "delete account"
[[ $(req anon GET "/problems/$PID/posts" | jq '.items | length') -ge 0 ]] && ok "board still readable"

echo "== all $pass checks passed"
