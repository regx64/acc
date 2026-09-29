import sys
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
if n > 14: sys.exit(3)
cl = [(int(d[2 + 2 * i]), int(d[3 + 2 * i])) for i in range(m)]
for mask in range(1 << n):
    t = lambda l: (mask >> (abs(l) - 1) & 1) == (l > 0)
    if all(t(a) or t(b) for a, b in cl):
        print(1); break
else:
    print(0)
