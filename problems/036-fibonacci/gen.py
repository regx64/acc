BRUTE_MAX_INPUT = 100000

SAMPLES = ["4\n0\n1\n10\n100\n"]

def tests(rng):
    out = ["1\n1000000000000000000", "30\n" + "\n".join(str(i) for i in range(30))]
    out.append("10000\n" + "\n".join(str(rng.randint(0, 10**18)) for _ in range(10000)))
    out.append("10000\n" + "\n".join(str(rng.randint(0, 2000)) for _ in range(10000)))
    return out
