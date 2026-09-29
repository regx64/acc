#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m; scanf("%d %d", &n, &m);
    vector<array<long long, 3>> es(m);
    for (auto& e : es) scanf("%lld %lld %lld", &e[0], &e[1], &e[2]);
    const long long INF = LLONG_MAX / 4;
    vector<long long> d(n + 1, INF); d[1] = 0;
    bool changed = false;
    for (int it = 0; it < n; it++) {
        changed = false;
        for (auto& e : es)
            if (d[e[0]] < INF && d[e[0]] + e[2] < d[e[1]]) { d[e[1]] = d[e[0]] + e[2]; changed = true; }
        if (!changed) break;
    }
    if (changed) { puts("-1"); return 0; }
    for (int i = 2; i <= n; i++) { if (d[i] >= INF) puts("-"); else printf("%lld\n", d[i]); }
}
