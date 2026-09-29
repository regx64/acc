import sys
d = sys.stdin.read().split()
n, k = int(d[0]), int(d[1]); t = list(map(int, d[2:2 + n]))
print(*[max(t[i:i + k]) for i in range(n - k + 1)])
