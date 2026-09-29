import sys
d = sys.stdin.read().split()
ns = list(map(int, d[1:]))
if max(ns) > 5000: sys.exit(3)
f = [0, 1]
for _ in range(max(ns)): f.append((f[-1] + f[-2]) % 1_000_000_007)
for n in ns: print(f[n])
