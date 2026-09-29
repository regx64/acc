SAMPLES = ["4 5\n1 2 3\n1 3 10\n2 3 4\n3 4 1\n4 1 5\n", "3 1\n2 3 1\n"]

def case(rng, n, m, tmax):
    es = [f"{rng.randint(1, n)} {rng.randint(1, n)} {rng.randint(0, tmax)}" for _ in range(m)]
    return f"{n} {m}" + ("\n" + "\n".join(es) if es else "")

def chain(n):
    es = [f"{i} {i + 1} 1000000000" for i in range(1, n)]
    return f"{n} {n - 1}\n" + "\n".join(es)

def tests(rng):
    out = ["1 0", "2 1\n1 1 5"]
    out += [case(rng, rng.randint(2, 30), rng.randint(0, 60), 20) for _ in range(6)]
    out += [case(rng, 100000, 300000, 10**9), case(rng, 100000, 300000, 10), case(rng, 100000, 150000, 10**9)]
    out.append(chain(100000))
    return out
