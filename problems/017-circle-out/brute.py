n, k = map(int, input().split())
alive = [True] * (n + 1)
cur = 0  # index of the last position counted
left = n
out = []
pos = n  # start before person 1
while left:
    steps = (k - 1) % left + 1
    while steps:
        pos = pos % n + 1
        if alive[pos]:
            steps -= 1
    alive[pos] = False
    out.append(pos)
    left -= 1
print(*out)
