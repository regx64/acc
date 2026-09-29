SAMPLES = ["5 4\n1 2 3 4 5\n2 1 5\n1 2 4 10\n2 1 3\n2 4 5\n"]

def case(rng, n, q, padd=0.5):
    a = [rng.randint(-10**9, 10**9) for _ in range(n)]
    cmds = []
    for _ in range(q):
        l = rng.randint(1, n); r = rng.randint(l, n)
        if rng.random() < padd:
            cmds.append(f"1 {l} {r} {rng.randint(-10**6, 10**6)}")
        else:
            cmds.append(f"2 {l} {r}")
    cmds[-1] = f"2 1 {n}"
    return f"{n} {q}\n" + " ".join(map(str, a)) + "\n" + "\n".join(cmds)

def tests(rng):
    out = ["1 2\n5\n1 1 1 -3\n2 1 1"]
    out += [case(rng, rng.randint(1, 12), 15) for _ in range(6)]
    out += [case(rng, 200000, 200000), case(rng, 200000, 200000, 0.9), case(rng, 200000, 200000, 0.1)]
    return out
