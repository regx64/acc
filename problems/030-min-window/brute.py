import sys
d = sys.stdin.read().split()
n, s = int(d[0]), int(d[1]); a = list(map(int, d[2:2 + n]))
best = 0
for L in range(1, n + 1):
    if any(sum(a[i:i + L]) >= s for i in range(n - L + 1)):
        best = L; break
print(best)
