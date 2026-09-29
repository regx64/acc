SAMPLES = ["3 4\n1 2 4\n1 3 3\n2 3 -1\n3 1 -2\n", "3 3\n1 2 4\n2 3 -5\n3 2 3\n", "3 1\n2 3 -7\n"]

def case(rng, n, m, lo, hi):
    es = [f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(lo, hi)}" for _ in range(m)]
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def dag_negative(rng, n, m):
    es = []
    for _ in range(m):
        a, b = sorted(rng.sample(range(1, n + 1), 2))
        es.append(f"{a} {b} {rng.randint(-10000, 10000)}")
    return f"{n} {m}\n" + "\n".join(es)

def unreachable_cycle(n):
    es = [f"1 2 5", f"{n - 1} {n} -3", f"{n} {n - 1} -3"]
    return f"{n} {len(es)}\n" + "\n".join(es)

def tests(rng):
    out = ["1 0", "2 1\n1 1 -1", "2 1\n2 2 -1"]
    out += [case(rng, rng.randint(2, 8), rng.randint(0, 14), -3, 10) for _ in range(6)]
    out += [dag_negative(rng, 500, 6000), dag_negative(rng, 500, 3000), case(rng, 500, 6000, -5, 10000), unreachable_cycle(500)]
    return out
