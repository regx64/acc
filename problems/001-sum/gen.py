SAMPLES = ["3 4\n", "-1000000000000 999999999999\n"]

def tests(rng):
    L = 10**12
    out = ["0 0", f"{L} {L}", f"{-L} {-L}", "1 -1"]
    out += [f"{rng.randint(-L, L)} {rng.randint(-L, L)}" for _ in range(8)]
    return out
