import sys
d = sys.stdin.read().split()
n = int(d[0])
if n > 16: sys.exit(3)
w = [0] + list(map(int, d[1:1 + n]))
parent = [0, 0] + list(map(int, d[1 + n:n + n]))
best = 0
for mask in range(1 << n):
    chosen = [False] + [bool(mask >> i & 1) for i in range(n)]
    if all(not (chosen[v] and chosen[parent[v]]) for v in range(2, n + 1)):
        best = max(best, sum(w[v] for v in range(1, n + 1) if chosen[v]))
print(best)
