SAMPLES = ["4 5\n1 2 1\n2 3 4\n3 4 2\n1 4 3\n1 3 5\n", "3 1\n1 2 7\n"]

def connected(rng, n, m, cmax):
    es = [f"{i} {rng.randint(1, i - 1)} {rng.randint(1, cmax)}" for i in range(2, n + 1)]
    es += [f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, cmax)}" for _ in range(m - (n - 1))]
    rng.shuffle(es)
    return f"{n} {len(es)}" + ("\n" + "\n".join(es) if es else "")

def rand(rng, n, m, cmax):
    es = [f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(1, cmax)}" for _ in range(m)]
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def tests(rng):
    out = ["1 0", "2 0", "2 2\n1 1 5\n1 2 3"]
    out += [connected(rng, rng.randint(2, 8), 12, 20) for _ in range(4)]
    out += [rand(rng, 7, 6, 10)]
    out += [connected(rng, 100000, 200000, 10**9), connected(rng, 100000, 200000, 5), rand(rng, 100000, 200000, 10**9)]
    return out
