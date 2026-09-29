SAMPLES = ["8 3\nthe cat and the dog and the bird\n"]
L = "abcdefghijklmnopqrstuvwxyz"

def words(rng, pool, n):
    vocab = ["".join(rng.choice(L[:4]) for _ in range(rng.randint(1, 4))) for _ in range(pool)]
    ws = []
    for _ in range(n):
        # Zipf-like: earlier words are more common.
        ws.append(vocab[min(int(rng.paretovariate(1.2)) - 1, pool - 1)])
    return ws

def fmt(ws, k, rng):
    lines, line = [], []
    for w in ws:
        line.append(w)
        if len(line) >= rng.randint(1, 20):
            lines.append(" ".join(line)); line = []
    if line:
        lines.append(" ".join(line))
    return f"{len(ws)} {k}\n" + "\n".join(lines)

def tests(rng):
    out = ["1 5\nsolo", "4 2\nb a b a"]
    out.append(fmt(words(rng, 30, 100), 10, rng))
    out.append(fmt(words(rng, 300, 200000), 1000, rng))
    out.append(fmt(words(rng, 5, 200000), 1000, rng))
    out.append(fmt(["".join(rng.choice(L) for _ in range(20)) for _ in range(200000)], 1000, rng))
    return out
