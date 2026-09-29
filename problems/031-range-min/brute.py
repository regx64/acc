import sys
d = sys.stdin.read().split()
n, q = int(d[0]), int(d[1]); p = list(map(int, d[2:2 + n])); k = 2 + n
for _ in range(q):
    c, a, b = int(d[k]), int(d[k + 1]), int(d[k + 2]); k += 3
    if c == 1: p[a - 1] = b
    else: print(min(p[a - 1:b]))
