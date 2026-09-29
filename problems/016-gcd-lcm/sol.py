import sys
from math import gcd
d = sys.stdin.buffer.read().split()
t = int(d[0])
out = []
for i in range(t):
    a, b = int(d[1 + 2 * i]), int(d[2 + 2 * i])
    g = gcd(a, b)
    out.append(f"{g} {a // g * b}")
sys.stdout.write("\n".join(out) + "\n")
