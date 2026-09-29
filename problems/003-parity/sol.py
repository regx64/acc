import sys
data = sys.stdin.read().split()
n = int(data[0])
even = sum(1 for x in data[1:n + 1] if int(x) % 2 == 0)
print(even)
print(n - even)
