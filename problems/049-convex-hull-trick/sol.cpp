#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
// dp[i] = min_{j<=i} dp[j-1] + (x_i - x_j)^2 + C, with dp[0] = 0 (1-indexed points).
// = x_i^2 + C + min_j ( -2 x_j * x_i + x_j^2 + dp[j-1] ): lines with decreasing slope, queries increasing.
struct L { ll m, b; ll at(ll x) const { return m * x + b; } };
bool bad(const L& a, const L& b, const L& c) {
    // b is useless if intersection(a,c) is left of intersection(a,b)
    return (__int128)(c.b - a.b) * (a.m - b.m) <= (__int128)(b.b - a.b) * (a.m - c.m);
}
int main() {
    int n; ll C; scanf("%d %lld", &n, &C);
    vector<ll> x(n + 1);
    for (int i = 1; i <= n; i++) scanf("%lld", &x[i]);
    vector<ll> dp(n + 1, 0);
    deque<L> hull;
    size_t ptr = 0;
    vector<L> lines;
    for (int i = 1; i <= n; i++) {
        L nl{-2 * x[i], x[i] * x[i] + dp[i - 1]};
        while (lines.size() >= ptr + 2 && bad(lines[lines.size() - 2], lines.back(), nl)) lines.pop_back();
        lines.push_back(nl);
        if (ptr >= lines.size()) ptr = lines.size() - 1;
        while (ptr + 1 < lines.size() && lines[ptr + 1].at(x[i]) <= lines[ptr].at(x[i])) ptr++;
        dp[i] = lines[ptr].at(x[i]) + x[i] * x[i] + C;
    }
    printf("%lld\n", dp[n]);
}
