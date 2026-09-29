import sys
lines = sys.stdin.read().split()
for s in lines[1:]:
    prev = None
    while prev != s:
        prev = s
        s = s.replace("()", "").replace("[]", "").replace("{}", "")
    print("YES" if s == "" else "NO")
