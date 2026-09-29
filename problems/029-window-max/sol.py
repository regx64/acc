import sys
from collections import deque
d = sys.stdin.buffer.read().split()
n, k = int(d[0]), int(d[1])
t = list(map(int, d[2:2 + n]))
dq = deque()
out = []
for i, x in enumerate(t):
    while dq and t[dq[-1]] <= x:
        dq.pop()
    dq.append(i)
    if dq[0] <= i - k:
        dq.popleft()
    if i >= k - 1:
        out.append(t[dq[0]])
sys.stdout.write(" ".join(map(str, out)) + "\n")
