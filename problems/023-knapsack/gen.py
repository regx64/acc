BRUTE_MAX_INPUT = 300

SAMPLES = ["4 7\n6 13\n4 8\n3 6\n5 12\n"]

def case(rng, n, w, wmax, vmax):
    return f"{n} {w}\n" + "\n".join(f"{rng.randint(1, wmax)} {rng.randint(0, vmax)}" for _ in range(n))

def tests(rng):
    out = ["1 1\n2 5", "1 5\n5 7"]
    out += [case(rng, rng.randint(1, 12), rng.randint(1, 50), 20, 100) for _ in range(6)]
    out += [case(rng, 100, 100000, 100000, 10**9), case(rng, 100, 100000, 3000, 10**9), case(rng, 100, 100000, 2000, 1)]
    return out
