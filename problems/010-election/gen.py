SAMPLES = ["5\njiwoo\nminseo\njiwoo\nhana\nminseo\n", "1\nsolo\n"]

def name(rng):
    return "".join(rng.choice("abcde") for _ in range(rng.randint(1, 3)))

def tests(rng):
    out = ["2\nb\na", "3\nzz\nzz\na"]
    for n, pool in ((100, 5), (10000, 50), (100000, 1000), (100000, 3)):
        names = [name(rng) for _ in range(pool)]
        out.append(f"{n}\n" + "\n".join(rng.choice(names) for _ in range(n)))
    # Many distinct names, several tied at the top.
    names = ["".join(rng.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(10)) for _ in range(50000)]
    out.append("100000\n" + "\n".join(names + names))
    return out
