import sys
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
es = [(int(d[2 + 3 * i]), int(d[3 + 3 * i]), int(d[4 + 3 * i])) for i in range(m)]
# Prim on an adjacency matrix of cheapest edges.
INF = float("inf")
w = [[INF] * (n + 1) for _ in range(n + 1)]
for u, v, c in es:
    if u != v:
        w[u][v] = w[v][u] = min(w[u][v], c)
inside = [False] * (n + 1); best = [INF] * (n + 1); best[1] = 0; total = 0
for _ in range(n):
    u = min((i for i in range(1, n + 1) if not inside[i]), key=lambda i: best[i])
    if best[u] == INF:
        print(-1); sys.exit()
    inside[u] = True; total += best[u]
    for v in range(1, n + 1):
        if not inside[v] and w[u][v] < best[v]:
            best[v] = w[u][v]
print(total)
