SAMPLES = ["4\n1\n2\n10\n100\n"]

def tests(rng):
    out = ["3\n10000000\n9999991\n9999990"]
    out.append("100000\n" + "\n".join(str(rng.randint(1, 10**7)) for _ in range(100000)))
    out.append("50\n" + "\n".join(str(n) for n in range(1, 51)))
    return out
