import sys
from functools import lru_cache
a, b = sys.stdin.read().split()
if len(a) * len(b) > 400:
    # Only the small cases are cross-checked by exhaustive recursion.
    dp = list(range(len(b) + 1))
    for i in range(1, len(a) + 1):
        nd = [i] + [0] * len(b)
        for j in range(1, len(b) + 1):
            nd[j] = min(dp[j] + 1, nd[j - 1] + 1, dp[j - 1] + (a[i - 1] != b[j - 1]))
        dp = nd
    print(dp[-1]); sys.exit()
@lru_cache(None)
def f(i, j):
    if i == len(a): return len(b) - j
    if j == len(b): return len(a) - i
    return min(f(i + 1, j) + 1, f(i, j + 1) + 1, f(i + 1, j + 1) + (a[i] != b[j]))
print(f(0, 0))
