import sys
t, p = sys.stdin.read().split()
m = len(p)
fail = [0] * m
k = 0
for i in range(1, m):
    while k and p[i] != p[k]:
        k = fail[k - 1]
    if p[i] == p[k]:
        k += 1
    fail[i] = k
count, first, k = 0, -1, 0
for i, c in enumerate(t):
    while k and c != p[k]:
        k = fail[k - 1]
    if c == p[k]:
        k += 1
    if k == m:
        count += 1
        if first < 0:
            first = i - m + 2
        k = fail[k - 1]
print(count, first)
