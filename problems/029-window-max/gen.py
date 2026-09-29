SAMPLES = ["8 3\n1 3 -1 -3 5 3 6 7\n"]

def case(rng, n, k, lo, hi):
    return f"{n} {k}\n" + " ".join(str(rng.randint(lo, hi)) for _ in range(n))

def tests(rng):
    out = ["1 1\n-5", "3 3\n1 2 3", "5 1\n5 4 3 2 1"]
    out += [case(rng, rng.randint(1, 20), 1, -5, 5) for _ in range(2)]
    out += [case(rng, 30, rng.randint(1, 30), -10, 10) for _ in range(3)]
    out += [case(rng, 500000, 1000, -10**9, 10**9), case(rng, 500000, 250000, -10**9, 10**9), case(rng, 500000, 7, 0, 3)]
    out.append("500000 500\n" + " ".join(str(500000 - i) for i in range(500000)))
    return out
