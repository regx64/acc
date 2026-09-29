import sys
lines = sys.stdin.read().split("\n")
t = int(lines[0])
for i in range(1, t + 1):
    print(f"Case #{i}: " + " ".join(reversed(lines[i].split())))
