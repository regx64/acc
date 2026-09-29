SAMPLES = ["5 6\n1 5 3 3 2\n", "4 0\n0 0 0 0\n"]

def case(rng, n, lo, hi, k=None):
    a = [rng.randint(lo, hi) for _ in range(n)]
    if k is None:
        k = rng.choice(a) + rng.choice(a)
    return f"{n} {k}\n" + " ".join(map(str, a))

def tests(rng):
    out = ["2 3\n1 2", "2 5\n1 2", case(rng, 30, -5, 5), case(rng, 60, -3, 3, 0)]
    out += [case(rng, 200000, -10**9, 10**9) for _ in range(2)]
    out += [case(rng, 200000, -1000, 1000), case(rng, 200000, 0, 1, 1)]
    out.append("200000 2000000000\n" + " ".join(["1000000000"] * 200000))
    return out
