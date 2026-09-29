import sys
d = sys.stdin.buffer.read().split()
n = int(d[0])
pts = sorted(set((int(d[1 + 2 * i]), int(d[2 + 2 * i])) for i in range(n)))
def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
def half(ps):
    h = []
    for p in ps:
        while len(h) >= 2 and cross(h[-2], h[-1], p) <= 0:
            h.pop()
        h.append(p)
    return h
lower, upper = half(pts), half(reversed(pts))
hull = lower[:-1] + upper[:-1]
area2 = 0
for i in range(len(hull)):
    x1, y1 = hull[i]; x2, y2 = hull[(i + 1) % len(hull)]
    area2 += x1 * y2 - x2 * y1
print(len(hull), abs(area2))
