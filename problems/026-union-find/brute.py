import sys
d = sys.stdin.read().split()
n, q = int(d[0]), int(d[1])
club = list(range(n + 1))
for i in range(q):
    c, a, b = d[2 + 3 * i], int(d[3 + 3 * i]), int(d[4 + 3 * i])
    if c == "U":
        old, new = club[b], club[a]
        club = [new if x == old else x for x in club]
    else:
        print("YES" if club[a] == club[b] else "NO")
from collections import Counter
print(max(Counter(club[1:]).values()))
