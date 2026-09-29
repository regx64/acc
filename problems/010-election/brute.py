import sys
data = sys.stdin.read().split()
votes = data[1:]
best_name, best = None, -1
for name in sorted(set(votes)):
    k = votes.count(name)
    if k > best:
        best_name, best = name, k
print(best_name, best)
