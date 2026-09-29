import sys
from itertools import product
# Enumerate integer flows on tiny graphs.
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
es = [tuple(int(x) for x in d[2 + 4 * i:6 + 4 * i]) for i in range(m)]
if m > 6 or n > 6: sys.exit(3)
best = (0, 0)
for f in product(*[range(c + 1) for _, _, c, _ in es]):
    bal = [0] * (n + 1)
    for (u, v, _, _), x in zip(es, f):
        bal[u] -= x; bal[v] += x
    if all(bal[i] == 0 for i in range(2, n)):
        flow = bal[n]
        cost = sum(x * w for (_, _, _, w), x in zip(es, f))
        if flow > best[0] or (flow == best[0] and cost < best[1]):
            best = (flow, cost)
print(*best)
