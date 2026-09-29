SAMPLES = ["ACGACGTACG\nACG\n", "AAAA\nAA\n", "ACGT\nTT\n"]

def rs(rng, n, alpha="ACGT"):
    return "".join(rng.choice(alpha) for _ in range(n))

def tests(rng):
    out = ["A\nA", "A\nAA", "GATTACA\nGATTACA"]
    for _ in range(5):
        out.append(f"{rs(rng, rng.randint(1, 40), 'AC')}\n{rs(rng, rng.randint(1, 3), 'AC')}")
    out.append("A" * 1000000 + "\n" + "A" * 1000)
    out.append("A" * 1000000 + "\n" + "A" * 999 + "C")
    t = rs(rng, 1000000); out.append(f"{t}\n{t[500000:500010]}")
    out.append(("ACGT" * 250000) + "\n" + "ACGTACGTAC")
    out.append(rs(rng, 1000000, "AC") + "\n" + rs(rng, 12, "AC"))
    return out
