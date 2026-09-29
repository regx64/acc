SAMPLES = ["4 5\n1 2 3\n1 3 2\n2 3 1\n2 4 2\n3 4 3\n", "3 1\n2 3 5\n"]

def case(rng, n, m, cmax):
    es = []
    for _ in range(m):
        u, v = rng.sample(range(1, n + 1), 2)
        es.append(f"{u} {v} {rng.randint(1, cmax)}")
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def layered(rng, n, cmax):
    layers = [[1]] + [list(range(2 + i * 10, 12 + i * 10)) for i in range((n - 2) // 10)] + [[n]]
    es = []
    for a, b in zip(layers, layers[1:]):
        for u in a:
            for v in rng.sample(b, min(len(b), 4)):
                es.append(f"{u} {v} {rng.randint(1, cmax)}")
    return f"{n} {len(es)}\n" + "\n".join(es)

def tests(rng):
    out = ["2 0", "2 1\n1 2 1000000000", "2 2\n1 2 5\n2 1 7"]
    out += [case(rng, rng.randint(2, 7), rng.randint(0, 12), 10) for _ in range(6)]
    out += [case(rng, 500, 5000, 10**9), case(rng, 500, 5000, 3), layered(rng, 492, 10**6)]
    return out
