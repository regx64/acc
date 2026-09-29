import sys
data = sys.stdin.read().split()
n = int(data[0])
h = list(map(int, data[1:n + 1]))
m = max(h)
print(m, h.index(m) + 1)
