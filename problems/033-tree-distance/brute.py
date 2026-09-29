import sys
d = sys.stdin.read().split()
n = int(d[0]); par = [0, 0] + list(map(int, d[1:n]))
k = n; q = int(d[k]); k += 1
def anc(x):
    out = [x]
    while x != 1:
        x = par[x]; out.append(x)
    return out
for _ in range(q):
    a, b = int(d[k]), int(d[k + 1]); k += 2
    A, B = anc(a), anc(b)
    sb = set(B)
    for i, x in enumerate(A):
        if x in sb:
            print(i + B.index(x), x); break
