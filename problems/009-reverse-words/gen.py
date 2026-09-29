SAMPLES = ["2\nhello world\nthis is acc\n", "1\nsolo\n"]

def word(rng):
    return "".join(rng.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(rng.randint(1, 8)))

def sentence(rng, max_len):
    words = [word(rng)]
    while True:
        w = word(rng)
        if len(" ".join(words)) + 1 + len(w) > max_len:
            break
        words.append(w)
        if rng.random() < 0.05:
            break
    return " ".join(words)

def tests(rng):
    out = ["1\na", "3\na b\nb a\nabc"]
    for _ in range(5):
        t = 100
        out.append(f"{t}\n" + "\n".join(sentence(rng, rng.choice([20, 200, 1000])) for _ in range(t)))
    return out
