import sys
d = sys.stdin.buffer.read().split()
n, s = int(d[0]), int(d[1])
a = list(map(int, d[2:2 + n]))
best = n + 1
total = 0
lo = 0
for hi, x in enumerate(a):
    total += x
    while total - a[lo] >= s:
        total -= a[lo]; lo += 1
    if total >= s:
        best = min(best, hi - lo + 1)
print(best if best <= n else 0)
