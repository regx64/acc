import sys
d = sys.stdin.buffer.read().split()
n, b = int(d[0]), int(d[1])
broken = bytearray(n + 1)
for x in d[2:2 + b]:
    broken[int(x)] = 1
MOD = 1_000_000_007
a0, a1, a2 = 0, 0, 1  # ways[i-3], ways[i-2], ways[i-1]; ways[0] = 1
for i in range(1, n + 1):
    w = 0 if broken[i] else (a0 + a1 + a2) % MOD
    a0, a1, a2 = a1, a2, w
print(a2)
