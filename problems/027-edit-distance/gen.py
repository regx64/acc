SAMPLES = ["kitten\nsitting\n", "acc\nacc\n"]

def rs(rng, n, alpha):
    return "".join(rng.choice(alpha) for _ in range(n))

def mutate(rng, s, k):
    s = list(s)
    for _ in range(k):
        op = rng.randrange(3)
        i = rng.randrange(len(s) + (op == 0))
        if op == 0:
            s.insert(i, rng.choice("abc"))
        elif op == 1 and len(s) > 1:
            s.pop(min(i, len(s) - 1))
        elif s:
            s[min(i, len(s) - 1)] = rng.choice("abc")
    return "".join(s)

def tests(rng):
    out = ["a\nb", "a\naaa", "abc\ncba"]
    for _ in range(5):
        a = rs(rng, rng.randint(1, 9), "abc")
        out.append(f"{a}\n{mutate(rng, a, rng.randint(0, 5)) or 'a'}")
    for alpha in ("ab", "abcdefghijklmnopqrstuvwxyz"):
        out.append(f"{rs(rng, 3000, alpha)}\n{rs(rng, 3000, alpha)}")
    a = rs(rng, 3000, "abc")
    out.append(f"{a}\n{mutate(rng, a, 300)[:3000]}")
    out.append("a" * 3000 + "\n" + "b" * 1)
    return out
