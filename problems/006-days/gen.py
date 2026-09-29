SAMPLES = ["3\n2024 2\n2023 2\n2025 11\n", "2\n1900 2\n2000 2\n"]

def tests(rng):
    out = []
    special = [(y, m) for y in (1, 4, 100, 400, 1600, 1700, 2000, 2100, 10000) for m in range(1, 13)]
    out.append(f"{len(special)}\n" + "\n".join(f"{y} {m}" for y, m in special))
    for _ in range(4):
        qs = [(rng.randint(1, 10000), rng.randint(1, 12)) for _ in range(1000)]
        out.append("1000\n" + "\n".join(f"{y} {m}" for y, m in qs))
    return out
