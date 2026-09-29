import sys
from fractions import Fraction
d = sys.stdin.read().split()
n = int(d[0])
pts = list(set((int(d[1 + 2 * i]), int(d[2 + 2 * i])) for i in range(n)))
def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
# Gift wrapping; a point is a vertex if some line through it has all others strictly on one side.
start = min(pts)
hull = [start]
cur = start
while True:
    cand = None
    for p in pts:
        if p == cur: continue
        if cand is None: cand = p; continue
        c = cross(cur, cand, p)
        if c < 0 or (c == 0 and (p[0]-cur[0])**2 + (p[1]-cur[1])**2 > (cand[0]-cur[0])**2 + (cand[1]-cur[1])**2):
            cand = p
    cur = cand
    if cur == start: break
    hull.append(cur)
area2 = abs(sum(hull[i][0] * hull[(i + 1) % len(hull)][1] - hull[(i + 1) % len(hull)][0] * hull[i][1] for i in range(len(hull))))
print(len(hull), area2)
