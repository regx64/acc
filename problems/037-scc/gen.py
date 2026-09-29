SAMPLES = ["5 6\n1 2\n2 3\n3 1\n3 4\n4 5\n5 4\n", "3 0\n"]

def case(rng, n, m):
    es = [f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(m)]
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def cycle(n):
    es = [f"{i} {i % n + 1}" for i in range(1, n + 1)]
    return f"{n} {n}\n" + "\n".join(es)

def clusters(rng, n, k):
    es = []
    for start in range(1, n + 1, k):
        grp = list(range(start, min(n, start + k - 1) + 1))
        for a, b in zip(grp, grp[1:] + grp[:1]):
            es.append(f"{a} {b}")
        if start > 1:
            es.append(f"{start - 1} {start}")
    return f"{n} {len(es)}\n" + "\n".join(es)

def tests(rng):
    out = ["1 1\n1 1", "2 2\n1 2\n2 1"]
    out += [case(rng, rng.randint(2, 10), rng.randint(0, 15)) for _ in range(6)]
    out += [case(rng, 100000, 300000), case(rng, 100000, 120000), cycle(100000), clusters(rng, 100000, 37)]
    return out
