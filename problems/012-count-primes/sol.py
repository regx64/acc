import sys
data = sys.stdin.buffer.read().split()
t = int(data[0])
qs = list(map(int, data[1:1 + t]))
m = max(qs)
sieve = bytearray([1]) * (m + 1)
sieve[0] = 0
if m >= 1:
    sieve[1] = 0
i = 2
while i * i <= m:
    if sieve[i]:
        sieve[i * i::i] = bytes(len(range(i * i, m + 1, i)))
    i += 1
# Answer queries in increasing order, counting primes between consecutive
# query points, so memory stays at one byte per number.
ans = [0] * t
count = 0
prev = 0
for idx in sorted(range(t), key=qs.__getitem__):
    n = qs[idx]
    count += sieve.count(1, prev, n + 1)
    prev = n + 1
    ans[idx] = count
sys.stdout.write("\n".join(map(str, ans)) + "\n")
