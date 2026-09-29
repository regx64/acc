import sys
d = sys.stdin.buffer.read().split()
n = int(d[0])
w = [0] + list(map(int, d[1:1 + n]))
parent = [0, 0] + list(map(int, d[1 + n:n + n]))
children = [[] for _ in range(n + 1)]
for v in range(2, n + 1):
    children[parent[v]].append(v)
order = [1]
for u in order:
    order.extend(children[u])
take = w[:]
skip = [0] * (n + 1)
for u in reversed(order):
    p = parent[u]
    if p:
        take[p] += skip[u]
        skip[p] += max(take[u], skip[u])
print(max(take[1], skip[1]))
