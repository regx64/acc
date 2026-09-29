import sys
t, p = sys.stdin.read().split()
pos = [i + 1 for i in range(len(t) - len(p) + 1) if t[i:i + len(p)] == p]
print(len(pos), pos[0] if pos else -1)
