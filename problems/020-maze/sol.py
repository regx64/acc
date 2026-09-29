import sys
from collections import deque
data = sys.stdin.buffer.read().split()
n, m = int(data[0]), int(data[1])
g = b"".join(data[2:2 + n])
s, e = g.index(b"S"), g.index(b"E")
dist = [-1] * (n * m)
dist[s] = 0
q = deque([s])
while q:
    c = q.popleft()
    if c == e:
        break
    r, col = divmod(c, m)
    d = dist[c] + 1
    for nb, ok in ((c - m, r > 0), (c + m, r < n - 1), (c - 1, col > 0), (c + 1, col < m - 1)):
        if ok and dist[nb] < 0 and g[nb] != 35:
            dist[nb] = d
            q.append(nb)
print(dist[e])
