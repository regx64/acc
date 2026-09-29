# 공식 문제

acc의 공식 문제 50개입니다. 모두 이 저장소를 위해 새로 쓴 문제이며 지문은 CC BY-SA 4.0입니다.

## 폴더 구조

```
problems/001-sum/
  problem.toml    제목, 시간·메모리 제한, 제안 레벨, 정해 파일
  statement.md    "# 문제", "# 입력", "# 출력", "# 힌트"(선택) 섹션
  gen.py          SAMPLES(예제 입력 목록)와 tests(rng) 함수
  sol.py|sol.cpp  정해
  brute.py        (선택) 느린 풀이. 작은 테스트에서 정해와 결과를 비교
```

- 테스트 데이터는 저장소에 넣지 않고 `gen.py`로 만듭니다. 시드는 폴더 이름에서 정해지므로 언제 만들어도 같습니다.
- 출력은 정해를 돌려서 만듭니다. `brute.py`가 있으면 입력이 작은 테스트(기본 4000바이트 이하, `gen.py`의 `BRUTE_MAX_INPUT`으로 조정)에서 정해와 비교하고, 다르면 빌드가 실패합니다. `brute.py`가 감당할 수 없는 입력에서는 종료 코드 3으로 건너뜁니다.

## 만들기와 올리기

```bash
python3 scripts/problems.py build                 # build/problems/<폴더>/tests.zip
python3 scripts/problems.py upload --api https://api.<도메인>/api/v1 \
  --login regx64 --password '…' --publish         # 만들기/갱신 → 테스트 업로드 → 정해 검증 → 공개
```

`upload`는 제목으로 기존 문제를 찾아 갱신하고, 이미 공개된 문제는 건너뜁니다. `--publish`는 `problem.toml`의 레벨로 공개하므로, 공개 전에 레벨을 검토하세요(M7).

## 목록

| 폴더 | 제목 | 제안 레벨 | 다루는 내용 |
| --- | --- | --- | --- |
| 001-sum | 두 수의 합 | ℕ1 | 입출력, 64비트 정수 |
| 002-receipt | 사칙연산 영수증 | ℕ1 | 사칙연산 |
| 003-parity | 짝수와 홀수 | ℕ1 | 반복문, 음수 나머지 |
| 004-tallest | 가장 높은 탑 | ℕ2 | 최댓값 |
| 005-stairs | 계단 그리기 | ℕ2 | 출력 형식 |
| 006-days | 이번 달은 며칠 | ℕ3 | 윤년 |
| 007-alarm | 알람 맞추기 | ℕ3 | 시각 계산 |
| 008-digit-sum | 아주 긴 수의 자릿수 합 | ℕ3 | 문자열 |
| 009-reverse-words | 말 순서 뒤집기 | ℕ4 | 문자열 처리 |
| 010-election | 반장 선거 | ℕ5 | 해시 맵 |
| 011-range-sum | 적금 통장 구간 합 | ℤ1 | 누적 합 |
| 012-count-primes | 소수 세기 | ℤ2 | 에라토스테네스의 체 |
| 013-brackets | 괄호 짝 맞추기 | ℤ1 | 스택 |
| 014-pair-sum | 합이 K인 짝 | ℤ3 | 해시 |
| 015-count-leq | 놀이기구 키 제한 | ℤ2 | 정렬, 이분 탐색 |
| 016-gcd-lcm | 타일과 막대 | ℤ1 | 최대공약수 |
| 017-circle-out | 동그랗게 앉아 술래 뽑기 | ℤ3 | 시뮬레이션 |
| 018-word-freq | 단어 빈도표 | ℤ4 | 정렬 기준 |
| 019-islands | 섬의 개수 | ℤ4 | 너비 우선 탐색 |
| 020-maze | 미로 탈출 | ℤ5 | 최단 거리 BFS |
| 021-broken-stairs | 부서진 계단 | ℚ1 | DP |
| 022-lis | 점점 높아지는 발자국 | ℚ2 | LIS |
| 023-knapsack | 캠핑 배낭 싸기 | ℚ2 | 배낭 DP |
| 024-dijkstra | 택배 최단 경로 | ℚ3 | 다익스트라 |
| 025-topo | 수강 순서 정하기 | ℚ3 | 위상 정렬 |
| 026-union-find | 동아리 합치기 | ℚ2 | 유니온 파인드 |
| 027-edit-distance | 오타 고치기 | ℚ3 | 편집 거리 |
| 028-mst | 섬마을 전력망 | ℚ3 | 최소 신장 트리 |
| 029-window-max | 일주일 최고 기온 | ℚ2 | 덱 |
| 030-min-window | 가장 짧은 저축 기간 | ℚ1 | 두 포인터 |
| 031-range-min | 주식 최저가 조회 | ℝ1 | 세그먼트 트리 |
| 032-tree-diameter | 가장 먼 두 마을 | ℝ1 | 트리 지름 |
| 033-tree-distance | 가계도 촌수 계산 | ℝ3 | LCA |
| 034-pattern-count | 유전자 서열 검색 | ℝ2 | KMP |
| 035-tsp | 배달 로봇 순회 | ℝ4 | 비트마스크 DP |
| 036-fibonacci | 토끼 개체 수 예측 | ℝ1 | 분할 정복 거듭제곱 |
| 037-scc | 서로 오갈 수 있는 도시들 | ℝ3 | 강한 연결 요소 |
| 038-matching | 멘토 짝 짓기 | ℝ4 | 이분 매칭 |
| 039-negative-cycle | 시간 여행 포털 | ℝ3 | 벨만-포드 |
| 040-inversions | 뒤바뀐 순위 | ℝ2 | 펜윅 트리 |
| 041-max-flow | 송수관 최대 유량 | ℂ2 | 최대 유량 |
| 042-convex-hull | 울타리 두르기 | ℂ1 | 볼록 껍질 |
| 043-range-add-sum | 구간 보너스 지급 | ℝ5 | 구간 갱신 |
| 044-distinct-substrings | 서로 다른 부분 문자열 | ℂ4 | 접미사 오토마톤 |
| 045-tree-party | 사내 파티 초대장 | ℝ2 | 트리 DP |
| 046-min-cost-flow | 택배 트럭 배차 | ℂ4 | 최소 비용 최대 유량 |
| 047-crt | 톱니바퀴 맞물림 | ℂ1 | 중국인의 나머지 정리 |
| 048-two-sat | 축제 부스 배치 | ℂ3 | 2-SAT |
| 049-convex-hull-trick | 고속도로 휴게소 공사 | ℍ1 | 볼록 껍질 최적화 |
| 050-running-median | 실시간 중간 점수 | ℚ4 | 힙 |
