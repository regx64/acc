SAMPLES = ["5 6\nU 1 2\nQ 1 3\nU 2 3\nQ 1 3\nU 4 4\nQ 5 4\n"]

def case(rng, n, q, pu):
    cmds = []
    for _ in range(q):
        c = "U" if rng.random() < pu else "Q"
        cmds.append(f"{c} {rng.randint(1, n)} {rng.randint(1, n)}")
    return f"{n} {q}\n" + "\n".join(cmds)

def chain(n):
    cmds = [f"U {i} {i + 1}" for i in range(1, n)] + [f"Q 1 {n}"]
    return f"{n} {len(cmds)}\n" + "\n".join(cmds)

def tests(rng):
    out = ["1 1\nQ 1 1", case(rng, 10, 30, 0.4), case(rng, 50, 100, 0.3)]
    out += [case(rng, 200000, 200000, p) for p in (0.3, 0.6, 0.9)]
    out.append(chain(199999))
    return out
