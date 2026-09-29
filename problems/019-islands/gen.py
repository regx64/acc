SAMPLES = ["4 5\n##..#\n#...#\n..#..\n.##.#\n", "1 1\n.\n"]

def grid(rng, n, m, p):
    return f"{n} {m}\n" + "\n".join("".join("#" if rng.random() < p else "." for _ in range(m)) for _ in range(n))

def snake(n, m):
    rows = []
    for i in range(n):
        if i % 2 == 0:
            rows.append("#" * m)
        elif (i // 2) % 2 == 0:
            rows.append("." * (m - 1) + "#")
        else:
            rows.append("#" + "." * (m - 1))
    return f"{n} {m}\n" + "\n".join(rows)

def tests(rng):
    out = ["1 1\n#", "2 2\n#.\n.#", grid(rng, 8, 9, 0.5), grid(rng, 30, 40, 0.4)]
    out += [grid(rng, 1000, 1000, p) for p in (0.3, 0.5, 0.6)]
    out.append(snake(1000, 1000))
    out.append(f"1000 1000\n" + "\n".join(["#" * 1000] * 1000))
    return out
