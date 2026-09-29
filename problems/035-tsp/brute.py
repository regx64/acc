import sys
from itertools import permutations
d = sys.stdin.read().split()
n = int(d[0]); c = [list(map(int, d[1 + i * n:1 + (i + 1) * n])) for i in range(n)]
if n > 8: sys.exit(3)
best = None
for p in permutations(range(1, n)):
    route = (0,) + p + (0,)
    if all(c[a][b] > 0 for a, b in zip(route, route[1:])):
        s = sum(c[a][b] for a, b in zip(route, route[1:]))
        best = s if best is None else min(best, s)
print(-1 if best is None else best)
