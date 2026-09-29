SAMPLES = ["10 15\n5 1 3 5 10 7 4 9 2 8\n", "3 100\n1 2 3\n"]

def case(rng, n, amax, s=None):
    a = [rng.randint(1, amax) for _ in range(n)]
    if s is None:
        s = rng.randint(1, max(1, sum(a) // rng.choice([1, 2, 10, 1000])))
    return f"{n} {s}\n" + " ".join(map(str, a))

def tests(rng):
    out = ["1 1\n1", "1 2\n1", "2 3\n1 2"]
    out += [case(rng, rng.randint(1, 30), 10) for _ in range(6)]
    out += [case(rng, 500000, 10**9) for _ in range(2)]
    out += [case(rng, 500000, 3, 10**15), case(rng, 500000, 1000, 1000000)]
    return out
