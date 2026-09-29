#include <bits/stdc++.h>
using namespace std;
struct E { int to; long long cap, cost; };
int main() {
    int n, m; scanf("%d %d", &n, &m);
    vector<E> es; vector<vector<int>> g(n + 1);
    for (int i = 0; i < m; i++) {
        int u, v; long long c, w; scanf("%d %d %lld %lld", &u, &v, &c, &w);
        g[u].push_back(es.size()); es.push_back({v, c, w});
        g[v].push_back(es.size()); es.push_back({u, 0, -w});
    }
    long long flow = 0, cost = 0;
    const long long INF = LLONG_MAX / 4;
    while (true) {
        // SPFA shortest path by cost (residual costs may be negative).
        vector<long long> d(n + 1, INF); vector<int> pe(n + 1, -1); vector<char> inq(n + 1, 0);
        deque<int> q; d[1] = 0; q.push_back(1);
        while (!q.empty()) {
            int u = q.front(); q.pop_front(); inq[u] = 0;
            for (int id : g[u]) {
                E& e = es[id];
                if (e.cap > 0 && d[u] + e.cost < d[e.to]) {
                    d[e.to] = d[u] + e.cost; pe[e.to] = id;
                    if (!inq[e.to]) { inq[e.to] = 1; q.push_back(e.to); }
                }
            }
        }
        if (d[n] >= INF) break;
        long long f = INF;
        for (int v = n; v != 1; v = es[pe[v] ^ 1].to) f = min(f, es[pe[v]].cap);
        for (int v = n; v != 1; v = es[pe[v] ^ 1].to) { es[pe[v]].cap -= f; es[pe[v] ^ 1].cap += f; }
        flow += f; cost += f * d[n];
    }
    printf("%lld %lld\n", flow, cost);
}
