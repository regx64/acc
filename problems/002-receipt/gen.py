SAMPLES = ["7 3\n", "2 5\n"]

def tests(rng):
    out = ["1 1", "1000000000 1", "1 1000000000", "1000000000 1000000000"]
    out += [f"{rng.randint(1, 10**9)} {rng.randint(1, 10**9)}" for _ in range(4)]
    out += [f"{rng.randint(1, 10**9)} {rng.randint(1, 1000)}" for _ in range(4)]
    return out
