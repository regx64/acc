import sys
d = sys.stdin.read().split()
n, q = int(d[0]), int(d[1])
h = list(map(int, d[2:2 + n]))
for x in map(int, d[2 + n:]):
    print(sum(1 for v in h if v <= x))
