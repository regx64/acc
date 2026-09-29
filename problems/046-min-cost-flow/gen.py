SAMPLES = ["4 5\n1 2 2 1\n1 3 1 5\n2 3 1 1\n2 4 1 3\n3 4 2 1\n", "3 1\n1 2 5 1\n"]

def case(rng, n, m, cmax, wmax):
    es = []
    for _ in range(m):
        u, v = rng.sample(range(1, n + 1), 2)
        es.append(f"{u} {v} {rng.randint(1, cmax)} {rng.randint(0, wmax)}")
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def tests(rng):
    out = ["2 0", "2 2\n1 2 3 5\n1 2 2 1"]
    out += [case(rng, rng.randint(2, 6), rng.randint(0, 10), 3, 10) for _ in range(6)]
    out += [case(rng, 100, 1000, 100, 1000), case(rng, 100, 1000, 1, 1000), case(rng, 100, 400, 100, 0)]
    return out
