import sys
d = sys.stdin.read().split()
n = int(d[0])
adj = [[] for _ in range(n + 1)]
for i in range(n - 1):
    u, v, w = int(d[1 + 3 * i]), int(d[2 + 3 * i]), int(d[3 + 3 * i])
    adj[u].append((v, w)); adj[v].append((u, w))
best = 0
for s in range(1, n + 1):
    dist = {s: 0}; st = [s]
    while st:
        u = st.pop()
        for v, w in adj[u]:
            if v not in dist: dist[v] = dist[u] + w; st.append(v)
    best = max(best, max(dist.values()))
print(best)
