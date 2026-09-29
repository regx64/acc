import sys
s = sys.stdin.read().split()[0]
if len(s) > 300: sys.exit(3)
print(len({s[i:j] for i in range(len(s)) for j in range(i + 1, len(s) + 1)}))
