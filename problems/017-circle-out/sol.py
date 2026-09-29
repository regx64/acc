n, k = map(int, input().split())
people = list(range(1, n + 1))
out = []
i = 0
while people:
    i = (i + k - 1) % len(people)
    out.append(people.pop(i))
print(*out)
