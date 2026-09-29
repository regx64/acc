SAMPLES = ["4\n([]){}\n([)]\n((\n}\n"]
PAIRS = ["()", "[]", "{}"]

def valid(rng, n):
    s = []
    stack = []
    while len(s) + len(stack) < n:
        if stack and (rng.random() < 0.5 or len(s) + len(stack) + 2 > n):
            s.append(stack.pop())
        else:
            p = rng.choice(PAIRS); s.append(p[0]); stack.append(p[1])
    s.extend(reversed(stack))
    return "".join(s)

def corrupt(rng, s):
    i = rng.randrange(len(s))
    c = rng.choice([c for c in "()[]{}" if c != s[i]])
    return s[:i] + c + s[i + 1:]

def tests(rng):
    out = ["3\n(\n)\n()", "2\n)(\n{[()]}"]
    for _ in range(3):
        strs = []
        for _ in range(1000):
            s = valid(rng, 2 * rng.randint(1, 500))
            strs.append(corrupt(rng, s) if rng.random() < 0.5 else s)
        out.append("1000\n" + "\n".join(strs))
    big = [valid(rng, 10000) for _ in range(100)]
    big = [corrupt(rng, s) if i % 3 == 0 else s for i, s in enumerate(big)]
    out.append("100\n" + "\n".join(big))
    out.append("2\n" + "(" * 5000 + ")" * 5000 + "\n" + "(" * 5000 + ")" * 4999 + "(")
    return out
