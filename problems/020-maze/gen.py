SAMPLES = ["3 4\nS..#\n.#..\n...E\n", "2 2\nS#\n#E\n"]

def maze(rng, n, m, p):
    g = [["#" if rng.random() < p else "." for _ in range(m)] for _ in range(n)]
    cells = rng.sample(range(n * m), 2)
    g[cells[0] // m][cells[0] % m] = "S"
    g[cells[1] // m][cells[1] % m] = "E"
    return f"{n} {m}\n" + "\n".join("".join(r) for r in g)

def zigzag(n, m):
    g = []
    for i in range(n):
        if i % 2 == 0:
            g.append(["."] * m)
        else:
            row = ["#"] * m
            row[m - 1 if (i // 2) % 2 == 0 else 0] = "."
            g.append(row)
    g[0][0] = "S"; g[n - 1][m - 1 if (n - 1) // 2 % 2 == 0 else 0] = "E"
    return f"{n} {m}\n" + "\n".join("".join(r) for r in g)

def tests(rng):
    out = ["2 2\nSE\n..", maze(rng, 5, 6, 0.3), maze(rng, 30, 30, 0.35)]
    out += [maze(rng, 1000, 1000, p) for p in (0.2, 0.35, 0.45)]
    out.append(zigzag(999, 1000))
    return out
