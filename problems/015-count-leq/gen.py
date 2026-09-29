SAMPLES = ["5 3\n150 120 180 120 160\n120 155 200\n"]

def case(rng, n, q, hi):
    return f"{n} {q}\n" + " ".join(str(rng.randint(1, hi)) for _ in range(n)) + "\n" + " ".join(str(rng.randint(1, hi)) for _ in range(q))

def tests(rng):
    out = ["1 2\n5\n4 5", case(rng, 20, 20, 30), case(rng, 1000, 1000, 100)]
    out += [case(rng, 200000, 200000, 10**9) for _ in range(2)]
    out.append(case(rng, 200000, 200000, 10))
    return out
