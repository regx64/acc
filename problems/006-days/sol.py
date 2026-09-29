import sys
data = sys.stdin.read().split()
t = int(data[0])
out = []
for i in range(t):
    y, m = int(data[1 + 2 * i]), int(data[2 + 2 * i])
    leap = y % 400 == 0 or (y % 4 == 0 and y % 100 != 0)
    if m == 2:
        out.append(29 if leap else 28)
    elif m in (4, 6, 9, 11):
        out.append(30)
    else:
        out.append(31)
print("\n".join(map(str, out)))
