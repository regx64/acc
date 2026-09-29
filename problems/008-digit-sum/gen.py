SAMPLES = ["12345\n", "9000000000000000000000000000001\n"]

def tests(rng):
    out = ["1", "9", "10"]
    for n in (100, 10000, 1000000, 1000000):
        out.append(str(rng.randint(1, 9)) + "".join(rng.choice("0123456789") for _ in range(n - 1)))
    out.append("9" * 1000000)
    return out
