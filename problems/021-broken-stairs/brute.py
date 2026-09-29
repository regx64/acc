import sys
from functools import lru_cache
d = sys.stdin.read().split()
n, b = int(d[0]), int(d[1])
if n > 60:
    sys.exit(3)
broken = set(map(int, d[2:2 + b]))
sys.setrecursionlimit(10000)
@lru_cache(None)
def go(i):
    if i == n: return 1
    if i > n or i in broken: return 0
    return go(i + 1) + go(i + 2) + go(i + 3)
print(go(0) % 1_000_000_007)
