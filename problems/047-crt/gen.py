SAMPLES = ["3\n2 3\n3 5\n2 7\n", "2\n1 4\n2 6\n", "2\n1 4\n3 6\n"]

def consistent(rng, n, mmax):
    x = rng.randint(0, 10**30)
    ms = [rng.randint(1, mmax) for _ in range(n)]
    return f"{n}\n" + "\n".join(f"{x % m} {m}" for m in ms)

def random_case(rng, n, mmax):
    rows = []
    for _ in range(n):
        m = rng.randint(1, mmax); rows.append(f"{rng.randint(0, m - 1)} {m}")
    return f"{n}\n" + "\n".join(rows)

def tests(rng):
    out = ["1\n0 1", "1\n999999999 1000000000", "2\n0 2\n1 2"]
    out += [consistent(rng, rng.randint(1, 4), 12) for _ in range(4)]
    out += [random_case(rng, rng.randint(2, 3), 10) for _ in range(4)]
    out += [consistent(rng, 1000, 10**9), consistent(rng, 1000, 1000), random_case(rng, 1000, 10**9)]
    primes = [p for p in range(2, 8000) if all(p % q for q in range(2, int(p ** 0.5) + 1))][-1000:]
    x = rng.randint(0, 10**3000)
    out.append("1000\n" + "\n".join(f"{x % p} {p}" for p in primes))
    return out
