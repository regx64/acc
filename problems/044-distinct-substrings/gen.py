SAMPLES = ["aba\n", "aaaa\n"]

def rs(rng, n, alpha):
    return "".join(rng.choice(alpha) for _ in range(n))

def tests(rng):
    out = ["a", "ab", "zzzzzzzzzz"]
    out += [rs(rng, rng.randint(1, 60), "ab") for _ in range(6)]
    out += [rs(rng, 200000, "abcdefghijklmnopqrstuvwxyz"), rs(rng, 200000, "ab"), "a" * 200000, ("abc" * 70000)[:200000]]
    fib = ["a", "ab"]
    while len(fib[-1]) < 200000: fib.append(fib[-1] + fib[-2])
    out.append(fib[-1][:200000])
    return out
