import sys
d = sys.stdin.read().split()
n = int(d[0]); a = list(map(int, d[1:1 + n]))
best = [1] * n
for i in range(n):
    for j in range(i):
        if a[j] < a[i]:
            best[i] = max(best[i], best[j] + 1)
print(max(best))
