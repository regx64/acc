import sys
d = sys.stdin.read().split()
n = int(d[0]); s = list(map(int, d[1:1 + n]))
res = []
for k in range(1, n + 1):
    t = sorted(s[:k]); res.append(t[(k + 1) // 2 - 1])
print(*res)
