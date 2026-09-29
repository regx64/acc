SAMPLES = ["5\n0 0\n4 0\n4 4\n0 4\n2 2\n", "4\n0 0\n2 0\n4 0\n1 3\n"]

def case(rng, n, r):
    pts = [(rng.randint(-r, r), rng.randint(-r, r)) for _ in range(n)]
    pts[0], pts[1], pts[2] = (0, 0), (1, 0), (0, 1)
    rng.shuffle(pts)
    return f"{n}\n" + "\n".join(f"{x} {y}" for x, y in pts)

def square_grid(k):
    pts = [(x, y) for x in range(k) for y in range(k)]
    return f"{len(pts)}\n" + "\n".join(f"{x} {y}" for x, y in pts)

def tests(rng):
    out = ["3\n0 0\n1000000000 -1000000000\n-1000000000 1000000000", "5\n0 0\n0 0\n1 0\n0 1\n1 0"]
    out += [case(rng, rng.randint(3, 12), 4) for _ in range(6)]
    out += [case(rng, 200000, 10**9), case(rng, 200000, 10), square_grid(400)]
    return out
