SAMPLES = ["23:45\n30\n", "09:05\n0\n"]

def tests(rng):
    out = ["00:00\n0", "23:59\n1", "00:00\n1440", "12:34\n1000000000", "23:59\n1000000000"]
    for _ in range(8):
        out.append(f"{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}\n{rng.randint(0, 10**9)}")
    return out
