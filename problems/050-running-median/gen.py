SAMPLES = ["5\n5 2 8 1 9\n", "4\n1 1 1 1\n"]

def case(rng, n, lo, hi):
    return f"{n}\n" + " ".join(str(rng.randint(lo, hi)) for _ in range(n))

def tests(rng):
    out = ["1\n-7", "2\n3 1"]
    out += [case(rng, rng.randint(1, 30), -5, 5) for _ in range(6)]
    out += [case(rng, 300000, -10**9, 10**9), case(rng, 300000, 0, 3)]
    out.append("300000\n" + " ".join(str(i) for i in range(300000)))
    out.append("300000\n" + " ".join(str(-i if i % 2 else i) for i in range(300000)))
    return out
