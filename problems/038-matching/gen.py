SAMPLES = ["3 3 5\n1 1\n1 2\n2 1\n3 2\n3 3\n", "2 1 2\n1 1\n2 1\n"]

def case(rng, n, m, e):
    es = [f"{rng.randint(1, n)} {rng.randint(1, m)}" for _ in range(e)]
    return f"{n} {m} {e}" + ("\n" + "\n".join(es) if es else "")

def hard(n):
    # Staircase: mentor i can take students 1..i. Greedy choices go wrong.
    es = [f"{i} {j}" for i in range(1, n + 1) for j in range(1, i + 1)]
    return f"{n} {n} {len(es)}\n" + "\n".join(es)

def tests(rng):
    out = ["1 1 0", "1 1 1\n1 1"]
    out += [case(rng, rng.randint(1, 7), rng.randint(1, 7), rng.randint(0, 12)) for _ in range(6)]
    out += [case(rng, 2000, 2000, 200000), case(rng, 2000, 2000, 4000), case(rng, 2000, 500, 50000), hard(600)]
    return out
