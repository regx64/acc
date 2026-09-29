import sys
d = sys.stdin.read().split()
n = int(d[0]); a = list(map(int, d[1:1 + n]))
print(sum(1 for i in range(n) for j in range(i + 1, n) if a[i] > a[j]))
