SAMPLES = ["6\n1 1 2 2 3\n3\n4 5\n4 6\n1 6\n"]

def case(rng, n, q, shape="random"):
    perm = [1] + rng.sample(range(2, n + 1), n - 1)  # perm[i] = label of node i (root stays 1)
    parent = [0] * (n + 1)
    for v in range(1, n):
        if shape == "path":
            p = v - 1
        elif shape == "deep":
            p = rng.randint(max(0, v - 3), v - 1)
        else:
            p = rng.randint(0, v - 1)
        parent[perm[v]] = perm[p]
    qs = "\n".join(f"{rng.randint(1, n)} {rng.randint(1, n)}" for _ in range(q))
    return f"{n}\n" + " ".join(str(parent[i]) for i in range(2, n + 1)) + f"\n{q}\n" + qs

def tests(rng):
    out = ["1\n\n1\n1 1", "2\n1\n2\n1 2\n2 2"]
    out += [case(rng, rng.randint(2, 12), 10) for _ in range(6)]
    out += [case(rng, 100000, 100000), case(rng, 100000, 100000, "path"), case(rng, 100000, 100000, "deep")]
    return out
