SAMPLES = ["4\n0 10 15 20\n5 0 9 10\n6 13 0 12\n8 8 9 0\n", "3\n0 1 -1\n-1 0 1\n-1 -1 0\n"]

def case(rng, n, p_missing, cmax):
    rows = []
    for i in range(n):
        row = []
        for j in range(n):
            row.append(0 if i == j else (-1 if rng.random() < p_missing else rng.randint(1, cmax)))
        rows.append(" ".join(map(str, row)))
    return f"{n}\n" + "\n".join(rows)

def tests(rng):
    out = ["2\n0 5\n7 0", "2\n0 -1\n3 0"]
    out += [case(rng, rng.randint(2, 7), 0.3, 20) for _ in range(6)]
    out += [case(rng, 16, 0.0, 10**6), case(rng, 16, 0.5, 10**6), case(rng, 16, 0.85, 100), case(rng, 15, 0.1, 10)]
    return out
