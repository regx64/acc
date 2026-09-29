import sys
from collections import Counter
d = sys.stdin.read().split()
n, k = int(d[0]), int(d[1])
c = Counter(d[2:2 + n])
rows = sorted(c.items(), key=lambda kv: (-kv[1], kv[0]))[:k]
print("\n".join(f"{w} {v}" for w, v in rows))
