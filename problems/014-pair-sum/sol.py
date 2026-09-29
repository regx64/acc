import sys
from collections import Counter
d = sys.stdin.buffer.read().split()
n, k = int(d[0]), int(d[1])
seen = Counter()
ans = 0
for x in map(int, d[2:2 + n]):
    ans += seen[k - x]
    seen[x] += 1
print(ans)
