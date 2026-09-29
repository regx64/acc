import sys
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
es = [(int(d[2 + 3 * i]), int(d[3 + 3 * i]), int(d[4 + 3 * i])) for i in range(m)]
# reachable set, then check for a negative cycle via Floyd on reachable nodes
reach = {1}
changed = True
while changed:
    changed = False
    for u, v, _ in es:
        if u in reach and v not in reach: reach.add(v); changed = True
INF = float("inf")
dist = [[INF] * (n + 1) for _ in range(n + 1)]
for i in range(1, n + 1): dist[i][i] = 0
for u, v, t in es: dist[u][v] = min(dist[u][v], t)
for k in range(1, n + 1):
    for i in range(1, n + 1):
        if dist[i][k] == INF: continue
        for j in range(1, n + 1):
            if dist[i][k] + dist[k][j] < dist[i][j]: dist[i][j] = dist[i][k] + dist[k][j]
if any(dist[i][i] < 0 for i in reach):
    print(-1)
else:
    for i in range(2, n + 1): print("-" if dist[1][i] == INF else dist[1][i])
