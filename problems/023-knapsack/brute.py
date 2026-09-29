import sys
d = sys.stdin.read().split()
n, W = int(d[0]), int(d[1])
items = [(int(d[2 + 2 * i]), int(d[3 + 2 * i])) for i in range(n)]
best = 0
for mask in range(1 << n):
    w = sum(items[i][0] for i in range(n) if mask >> i & 1)
    if w <= W:
        best = max(best, sum(items[i][1] for i in range(n) if mask >> i & 1))
print(best)
