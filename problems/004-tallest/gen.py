SAMPLES = ["5\n3 9 2 9 1\n", "1\n42\n"]

def tests(rng):
    out = ["3\n1 1 1", "4\n5 4 3 2", "4\n2 3 4 5"]
    for n in (100, 10000, 200000):
        out.append(f"{n}\n" + " ".join(str(rng.randint(1, 10**9)) for _ in range(n)))
    out.append("200000\n" + " ".join(str(rng.randint(1, 3)) for _ in range(200000)))
    return out
