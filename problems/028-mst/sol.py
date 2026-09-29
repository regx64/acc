import sys
d = sys.stdin.buffer.read().split()
n, m = int(d[0]), int(d[1])
edges = sorted(((int(d[4 + 3 * i]), int(d[2 + 3 * i]), int(d[3 + 3 * i])) for i in range(m)))
parent = list(range(n + 1))
def find(x):
    while parent[x] != x:
        parent[x] = parent[parent[x]]
        x = parent[x]
    return x
total = 0; used = 0
for c, u, v in edges:
    ru, rv = find(u), find(v)
    if ru != rv:
        parent[ru] = rv
        total += c; used += 1
        if used == n - 1:
            break
print(total if used == n - 1 else -1)
