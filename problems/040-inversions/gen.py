SAMPLES = ["5\n3 1 4 1 5\n", "3\n1 2 3\n"]

def case(rng, n, hi):
    return f"{n}\n" + " ".join(str(rng.randint(1, hi)) for _ in range(n))

def tests(rng):
    out = ["1\n5", "2\n2 1", "3\n2 2 2"]
    out += [case(rng, rng.randint(1, 40), 10) for _ in range(6)]
    out += [case(rng, 300000, 10**9), case(rng, 300000, 5)]
    out.append("300000\n" + " ".join(str(300000 - i) for i in range(300000)))
    return out
