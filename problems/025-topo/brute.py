import sys
from itertools import permutations
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
es = [(int(d[2 + 2 * i]), int(d[3 + 2 * i])) for i in range(m)]
for p in permutations(range(1, n + 1)):
    pos = {v: i for i, v in enumerate(p)}
    if all(pos[a] < pos[b] for a, b in es):
        print(*p); break
else:
    print(-1)
