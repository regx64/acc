SAMPLES = ["5\n1 2 3 4 5\n", "4\n0 -3 -4 7\n"]

def tests(rng):
    out = ["1\n0", "1\n-1", "3\n-1000000000 999999999 1000000000"]
    for n in (10, 1000, 100000, 100000):
        out.append(f"{n}\n" + " ".join(str(rng.randint(-10**9, 10**9)) for _ in range(n)))
    out.append("100000\n" + " ".join(["7"] * 100000))
    return out
