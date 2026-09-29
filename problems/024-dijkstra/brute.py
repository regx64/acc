import sys
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
es = [(int(d[2 + 3 * i]), int(d[3 + 3 * i]), int(d[4 + 3 * i])) for i in range(m)]
INF = float("inf")
dist = [INF] * (n + 1); dist[1] = 0
for _ in range(n):
    for u, v, t in es:
        if dist[u] + t < dist[v]:
            dist[v] = dist[u] + t
for i in range(1, n + 1):
    print(-1 if dist[i] == INF else dist[i])
