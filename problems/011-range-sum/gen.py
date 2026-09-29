SAMPLES = ["5 3\n1 2 3 4 5\n1 5\n2 4\n3 3\n"]

def case(rng, n, q, hi):
    a = [rng.randint(0, hi) for _ in range(n)]
    qs = []
    for _ in range(q):
        l = rng.randint(1, n); r = rng.randint(l, n)
        qs.append(f"{l} {r}")
    return f"{n} {q}\n" + " ".join(map(str, a)) + "\n" + "\n".join(qs)

def tests(rng):
    out = ["1 1\n0\n1 1", "1 1\n1000000000\n1 1", case(rng, 10, 20, 10), case(rng, 1000, 1000, 10**9)]
    out += [case(rng, 200000, 200000, 10**9) for _ in range(3)]
    return out
