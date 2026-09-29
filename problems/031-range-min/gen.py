SAMPLES = ["5 5\n5 3 8 6 2\n2 1 3\n2 2 5\n1 5 9\n2 4 5\n2 1 5\n"]

def case(rng, n, q, hi, pupd=0.5):
    p = [rng.randint(1, hi) for _ in range(n)]
    cmds = []
    for _ in range(q):
        if rng.random() < pupd:
            cmds.append(f"1 {rng.randint(1, n)} {rng.randint(1, hi)}")
        else:
            l = rng.randint(1, n); r = rng.randint(l, n)
            cmds.append(f"2 {l} {r}")
    if not any(c.startswith("2") for c in cmds):
        cmds[-1] = f"2 1 {n}"
    return f"{n} {q}\n" + " ".join(map(str, p)) + "\n" + "\n".join(cmds)

def tests(rng):
    out = ["1 2\n7\n1 1 3\n2 1 1"]
    out += [case(rng, rng.randint(1, 15), 20, 20) for _ in range(6)]
    out += [case(rng, 200000, 200000, 10**9), case(rng, 200000, 200000, 10**9, 0.1), case(rng, 200000, 200000, 5, 0.9)]
    return out
