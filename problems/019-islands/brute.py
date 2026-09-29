import sys
sys.setrecursionlimit(100000)
d = sys.stdin.read().split()
n, m = int(d[0]), int(d[1])
g = [list(r) for r in d[2:2 + n]]
def dfs(i, j):
    if not (0 <= i < n and 0 <= j < m) or g[i][j] != "#": return 0
    g[i][j] = "."
    return 1 + dfs(i + 1, j) + dfs(i - 1, j) + dfs(i, j + 1) + dfs(i, j - 1)
sizes = [dfs(i, j) for i in range(n) for j in range(m) if g[i][j] == "#"]
print(len(sizes), max(sizes, default=0))
