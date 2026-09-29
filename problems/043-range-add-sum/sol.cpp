#include <bits/stdc++.h>
using namespace std;
// Two Fenwick trees for range add / range sum.
int n; vector<long long> b1, b2;
void add(vector<long long>& b, int i, long long x) { for (; i <= n; i += i & -i) b[i] += x; }
long long sum(vector<long long>& b, int i) { long long s = 0; for (; i > 0; i -= i & -i) s += b[i]; return s; }
void range_add(int l, int r, long long x) { add(b1, l, x); add(b1, r + 1, -x); add(b2, l, x * (l - 1)); add(b2, r + 1, -x * r); }
long long prefix(int i) { return sum(b1, i) * i - sum(b2, i); }
int main() {
    int q; scanf("%d %d", &n, &q);
    b1.assign(n + 2, 0); b2.assign(n + 2, 0);
    for (int i = 1; i <= n; i++) { long long a; scanf("%lld", &a); range_add(i, i, a); }
    string out;
    while (q--) {
        int c, l, r; scanf("%d %d %d", &c, &l, &r);
        if (c == 1) { long long x; scanf("%lld", &x); range_add(l, r, x); }
        else out += to_string(prefix(r) - prefix(l - 1)) + "\n";
    }
    fputs(out.c_str(), stdout);
}
