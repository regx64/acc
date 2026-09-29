import sys
from math import lcm
d = sys.stdin.read().split()
n = int(d[0]); eqs = [(int(d[1 + 2 * i]), int(d[2 + 2 * i])) for i in range(n)]
L = 1
for _, m in eqs: L = lcm(L, m)
if L > 10**6: sys.exit(3)
for x in range(L):
    if all(x % m == a for a, m in eqs):
        print(x); break
else:
    print(-1)
