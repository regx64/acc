import sys, heapq
d = sys.stdin.buffer.read().split()
n = int(d[0])
low, high = [], []  # low is a max-heap (negated), holds ceil(k/2) smallest
out = []
for x in map(int, d[1:1 + n]):
    if not low or x <= -low[0]:
        heapq.heappush(low, -x)
    else:
        heapq.heappush(high, x)
    if len(low) > len(high) + 1:
        heapq.heappush(high, -heapq.heappop(low))
    elif len(high) > len(low):
        heapq.heappush(low, -heapq.heappop(high))
    out.append(-low[0])
sys.stdout.write(" ".join(map(str, out)) + "\n")
