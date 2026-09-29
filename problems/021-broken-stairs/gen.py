SAMPLES = ["4 0\n", "5 1\n2\n", "4 2\n1 2\n"]

def case(rng, n, b):
    br = rng.sample(range(1, n), b) if b else []
    return f"{n} {b}" + ("\n" + " ".join(map(str, br)) if b else "")

def tests(rng):
    out = ["1 0", "3 2\n1 2", "7 3\n4 5 6"]
    out += [case(rng, rng.randint(2, 25), rng.randint(0, 4)) for _ in range(6)]
    out += [case(rng, 1000000, 0), case(rng, 1000000, 1000), case(rng, 1000000, 300000)]
    return out
