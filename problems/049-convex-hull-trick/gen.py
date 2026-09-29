BRUTE_MAX_INPUT = 20000

SAMPLES = ["5 10\n1 2 3 10 11\n", "1 7\n0\n"]

def case(rng, n, xmax, c):
    xs = sorted(rng.sample(range(0, xmax + 1), n))
    return f"{n} {c}\n" + " ".join(map(str, xs))

def tests(rng):
    out = ["2 0\n0 1000000", "3 1000000000000\n0 500000 1000000"]
    out += [case(rng, rng.randint(1, 12), 50, rng.randint(0, 200)) for _ in range(6)]
    out += [case(rng, 2000, 10**6, c) for c in (0, 10**5, 10**9, 10**12)]
    out += [case(rng, 300000, 10**6, 10**6), case(rng, 300000, 10**6, 0), case(rng, 300000, 10**6, 10**12)]
    out.append("300000 5\n" + " ".join(str(i) for i in range(300000)))
    return out
