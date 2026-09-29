import sys
from math import gcd
d = sys.stdin.read().split()
n = int(d[0])
x, m = 0, 1
for i in range(n):
    a, mi = int(d[1 + 2 * i]), int(d[2 + 2 * i])
    g = gcd(m, mi)
    if (a - x) % g:
        print(-1)
        sys.exit()
    # Solve x + m*t ≡ a (mod mi)  ->  (m/g)*t ≡ (a-x)/g (mod mi/g)
    mg = mi // g
    t = ((a - x) // g) * pow(m // g, -1, mg) % mg if mg > 1 else 0
    x += m * t
    m = m // g * mi
    x %= m
print(x)
