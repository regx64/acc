h, m = map(int, input().strip().split(":"))
k = int(input())
t = (h * 60 + m + k) % 1440
print(f"{t // 60:02d}:{t % 60:02d}")
