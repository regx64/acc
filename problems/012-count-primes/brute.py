import sys
d = sys.stdin.read().split()
qs = list(map(int, d[1:]))
m = max(qs)
is_p = [True] * (m + 1)
is_p[0] = False
if m >= 1:
    is_p[1] = False
for i in range(2, m + 1):
    if is_p[i] and i * i <= m:
        for j in range(i * i, m + 1, i):
            is_p[j] = False
cnt = [0] * (m + 1)
for i in range(1, m + 1):
    cnt[i] = cnt[i - 1] + is_p[i]
for n in qs:
    print(cnt[n])
