import sys
d = sys.stdin.read().split()
n, C = int(d[0]), int(d[1]); x = [0] + list(map(int, d[2:2 + n]))
if n > 2000: sys.exit(3)
dp = [0] * (n + 1)
for i in range(1, n + 1):
    dp[i] = min(dp[j - 1] + (x[i] - x[j]) ** 2 + C for j in range(1, i + 1))
print(dp[n])
