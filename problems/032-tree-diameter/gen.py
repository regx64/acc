SAMPLES = ["5\n1 2 3\n2 3 4\n2 4 2\n4 5 6\n"]

def tree(rng, n, wmax, shape="random"):
    es = []
    for v in range(2, n + 1):
        if shape == "path":
            p = v - 1
        elif shape == "star":
            p = 1
        else:
            p = rng.randint(max(1, v - 5) if shape == "deep" else 1, v - 1)
        es.append((p, v, rng.randint(1, wmax)))
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    return f"{n}\n" + "\n".join(f"{perm[a - 1]} {perm[b - 1]} {w}" for a, b, w in es)

def tests(rng):
    out = ["2\n1 2 7"]
    out += [tree(rng, rng.randint(2, 12), 10) for _ in range(6)]
    out += [tree(rng, 200000, 10**9), tree(rng, 200000, 10**9, "path"), tree(rng, 200000, 10**9, "star"), tree(rng, 200000, 100, "deep")]
    return out
