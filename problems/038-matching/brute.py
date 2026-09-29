import sys
from itertools import permutations
d = sys.stdin.read().split()
n, m, e = int(d[0]), int(d[1]), int(d[2])
if n > 7 or m > 7: sys.exit(3)
ok = {(int(d[3 + 2 * i]), int(d[4 + 2 * i])) for i in range(e)}
best = 0
# assign each mentor a student or nobody (0)
def go(i, used, cnt):
    global best
    if i > n:
        best = max(best, cnt); return
    go(i + 1, used, cnt)
    for b in range(1, m + 1):
        if b not in used and (i, b) in ok:
            go(i + 1, used | {b}, cnt + 1)
go(1, frozenset(), 0)
print(best)
