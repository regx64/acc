SAMPLES = ["6\n10 20 10 30 20 50\n", "3\n5 5 5\n"]

def case(rng, n, hi):
    return f"{n}\n" + " ".join(str(rng.randint(1, hi)) for _ in range(n))

def tests(rng):
    out = ["1\n7", "5\n5 4 3 2 1", "5\n1 2 3 4 5"]
    out += [case(rng, rng.randint(1, 12), 10) for _ in range(6)]
    out += [case(rng, 200000, 10**9), case(rng, 200000, 1000)]
    out.append("200000\n" + " ".join(str(i) for i in range(1, 200001)))
    out.append("200000\n" + " ".join(str(i // 2 + (i % 2) * 100000) for i in range(200000)))
    return out
