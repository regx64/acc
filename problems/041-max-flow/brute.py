import sys
from itertools import product
# Max-flow = min-cut: try every cut on small graphs.
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
if n > 12: sys.exit(3)
es = [(int(d[2 + 3 * i]), int(d[3 + 3 * i]), int(d[4 + 3 * i])) for i in range(m)]
best = None
for bits in product([0, 1], repeat=n - 2):
    side = {1: 0, n: 1}
    for i, b in enumerate(bits): side[i + 2] = b
    cut = sum(c for u, v, c in es if side[u] == 0 and side[v] == 1)
    best = cut if best is None else min(best, cut)
print(best)
