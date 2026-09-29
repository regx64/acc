SAMPLES = ["3 4\n1 2\n-1 3\n-2 -3\n1 -3\n", "1 2\n1 1\n-1 -1\n"]

def lit(rng, n):
    return rng.randint(1, n) * rng.choice((1, -1))

def planted(rng, n, m):
    val = [None] + [rng.random() < 0.5 for _ in range(n)]
    true = lambda l: val[abs(l)] == (l > 0)
    cl = []
    while len(cl) < m:
        a, b = lit(rng, n), lit(rng, n)
        if true(a) or true(b):
            cl.append(f"{a} {b}")
    return f"{n} {m}\n" + "\n".join(cl)

def random_case(rng, n, m):
    return f"{n} {m}\n" + "\n".join(f"{lit(rng, n)} {lit(rng, n)}" for _ in range(m))

def tests(rng):
    out = ["1 1\n1 -1", "2 4\n1 2\n-1 2\n1 -2\n-1 -2"]
    out += [random_case(rng, rng.randint(1, 6), rng.randint(1, 12)) for _ in range(6)]
    out += [planted(rng, 100000, 200000), random_case(rng, 100000, 200000), planted(rng, 1000, 200000), random_case(rng, 2000, 2000)]
    return out
