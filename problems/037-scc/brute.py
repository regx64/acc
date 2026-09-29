import sys
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
reach = [[i == j for j in range(n + 1)] for i in range(n + 1)]
for i in range(m):
    reach[int(d[2 + 2 * i])][int(d[3 + 2 * i])] = True
for k in range(1, n + 1):
    for i in range(1, n + 1):
        if reach[i][k]:
            for j in range(1, n + 1):
                if reach[k][j]: reach[i][j] = True
comps = {frozenset(j for j in range(1, n + 1) if reach[i][j] and reach[j][i]) for i in range(1, n + 1)}
print(len(comps), max(map(len, comps)))
