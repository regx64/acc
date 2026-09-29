SAMPLES = ["7 3\n", "5 1\n"]

def tests(rng):
    out = ["1 1", "1 1000000000", "2 2", "10 11"]
    out += [f"{rng.randint(1, 50)} {rng.randint(1, 60)}" for _ in range(4)]
    out += ["5000 5000", "5000 1000000000", "5000 2", "4999 123456789"]
    return out
