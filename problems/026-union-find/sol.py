import sys
d = sys.stdin.buffer.read().split()
n, q = int(d[0]), int(d[1])
parent = list(range(n + 1))
size = [1] * (n + 1)
def find(x):
    root = x
    while parent[root] != root:
        root = parent[root]
    while parent[x] != root:
        parent[x], x = root, parent[x]
    return root
out = []
p = 2
for _ in range(q):
    c, a, b = d[p], int(d[p + 1]), int(d[p + 2]); p += 3
    ra, rb = find(a), find(b)
    if c == b"U":
        if ra != rb:
            if size[ra] < size[rb]:
                ra, rb = rb, ra
            parent[rb] = ra
            size[ra] += size[rb]
    else:
        out.append("YES" if ra == rb else "NO")
out.append(str(max(size[find(i)] for i in range(1, n + 1))))
sys.stdout.write("\n".join(out) + "\n")
