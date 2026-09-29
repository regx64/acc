import sys
d = sys.stdin.buffer.read().split()
n = int(d[0])
a = list(map(int, d[1:1 + n]))
rank = {v: i + 1 for i, v in enumerate(sorted(set(a)))}
size = len(rank)
tree = [0] * (size + 1)
inv = 0
for i, x in enumerate(a):
    r = rank[x]
    # count of earlier values <= r
    s = 0; j = r
    while j > 0:
        s += tree[j]; j -= j & -j
    inv += i - s
    j = r
    while j <= size:
        tree[j] += 1; j += j & -j
print(inv)
