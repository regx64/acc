import sys
from bisect import bisect_right
d = sys.stdin.buffer.read().split()
n, q = int(d[0]), int(d[1])
h = sorted(map(int, d[2:2 + n]))
sys.stdout.write("\n".join(str(bisect_right(h, int(x))) for x in d[2 + n:2 + n + q]) + "\n")
