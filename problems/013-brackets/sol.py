import sys
lines = sys.stdin.read().split()
match = {")": "(", "]": "[", "}": "{"}
out = []
for s in lines[1:1 + int(lines[0])]:
    st = []
    ok = True
    for c in s:
        if c in "([{":
            st.append(c)
        elif not st or st.pop() != match[c]:
            ok = False
            break
    out.append("YES" if ok and not st else "NO")
print("\n".join(out))
