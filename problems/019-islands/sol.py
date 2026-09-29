import sys
from collections import deque
data = sys.stdin.buffer.read().split()
n, m = int(data[0]), int(data[1])
g = bytearray(b"".join(data[2:2 + n]))
count = best = 0
for s in range(n * m):
    if g[s] != 35:  # '#'
        continue
    count += 1
    g[s] = 46
    q = deque([s]); size = 0
    while q:
        c = q.popleft(); size += 1
        r, col = divmod(c, m)
        for nb, ok in ((c - m, r > 0), (c + m, r < n - 1), (c - 1, col > 0), (c + 1, col < m - 1)):
            if ok and g[nb] == 35:
                g[nb] = 46
                q.append(nb)
    best = max(best, size)
print(count, best)
