import sys
MOD = 1_000_000_007
def fib(n):
    # fast doubling: returns (F(n), F(n+1))
    if n == 0:
        return 0, 1
    a, b = fib(n >> 1)
    c = a * ((2 * b - a) % MOD) % MOD
    d = (a * a + b * b) % MOD
    return (d, (c + d) % MOD) if n & 1 else (c, d)
d = sys.stdin.read().split()
print("\n".join(str(fib(int(x))[0]) for x in d[1:1 + int(d[0])]))
