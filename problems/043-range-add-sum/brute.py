import sys
d = sys.stdin.read().split()
n, q = int(d[0]), int(d[1]); a = list(map(int, d[2:2 + n])); k = 2 + n
for _ in range(q):
    c = int(d[k])
    if c == 1:
        l, r, x = int(d[k + 1]), int(d[k + 2]), int(d[k + 3]); k += 4
        for i in range(l - 1, r): a[i] += x
    else:
        l, r = int(d[k + 1]), int(d[k + 2]); k += 3
        print(sum(a[l - 1:r]))
