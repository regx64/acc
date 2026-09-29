SAMPLES = ["5\n3 2 4 5 1\n1 1 2 2\n"]

def case(rng, n, wmax, shape="random"):
    perm = [1] + rng.sample(range(2, n + 1), n - 1)
    parent = [0] * (n + 1)
    for v in range(1, n):
        p = v - 1 if shape == "path" else rng.randint(max(0, v - 4) if shape == "deep" else 0, v - 1)
        parent[perm[v]] = perm[p]
    return f"{n}\n" + " ".join(str(rng.randint(0, wmax)) for _ in range(n)) + "\n" + " ".join(str(parent[i]) for i in range(2, n + 1))

def tests(rng):
    out = ["1\n7\n", "2\n5 6\n1"]
    out += [case(rng, rng.randint(2, 14), 10) for _ in range(6)]
    out += [case(rng, 200000, 10**9), case(rng, 200000, 10**9, "path"), case(rng, 200000, 3, "deep")]
    return out
