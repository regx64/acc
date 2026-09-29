import sys
from bisect import bisect_left
d = sys.stdin.buffer.read().split()
n = int(d[0])
tails = []
for x in map(int, d[1:1 + n]):
    i = bisect_left(tails, x)
    if i == len(tails):
        tails.append(x)
    else:
        tails[i] = x
print(len(tails))
