SAMPLES = ["4 3\n4 2\n3 1\n1 2\n", "3 3\n1 2\n2 3\n3 1\n"]

def dag(rng, n, m):
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    es = []
    for _ in range(m):
        i, j = sorted(rng.sample(range(n), 2))
        es.append(f"{perm[i]} {perm[j]}")
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def cyclic(rng, n, m):
    s = dag(rng, n, m - 1).split("\n")
    a, b = s[1].split()
    s[0] = f"{n} {m}"
    s.append(f"{b} {a}")
    return "\n".join(s)

def tests(rng):
    out = ["1 0", "2 0", "2 1\n2 1"]
    out += [dag(rng, rng.randint(2, 8), rng.randint(0, 8)) for _ in range(5)]
    out += [cyclic(rng, 6, 5)]
    out += [dag(rng, 100000, 200000), dag(rng, 100000, 1000), cyclic(rng, 100000, 200000)]
    return out
