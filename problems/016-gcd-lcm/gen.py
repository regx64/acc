SAMPLES = ["3\n12 18\n7 5\n6 6\n"]

def tests(rng):
    out = ["2\n1 1\n1000000000000 999999999999"]
    qs = []
    for _ in range(100000):
        g = rng.randint(1, 10**6)
        qs.append(f"{g * rng.randint(1, 10**6)} {g * rng.randint(1, 10**6)}")
    out.append("100000\n" + "\n".join(qs))
    out.append("100000\n" + "\n".join(f"{rng.randint(1, 10**12)} {rng.randint(1, 10**12)}" for _ in range(100000)))
    return out
