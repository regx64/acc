#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<vector<pair<int, long long>>> g(n + 1);
    for (int i = 0; i < m; i++) {
        int u, v; long long t;
        scanf("%d %d %lld", &u, &v, &t);
        g[u].push_back({v, t});
    }
    const long long INF = LLONG_MAX;
    vector<long long> d(n + 1, INF);
    priority_queue<pair<long long, int>, vector<pair<long long, int>>, greater<>> pq;
    d[1] = 0; pq.push({0, 1});
    while (!pq.empty()) {
        auto [du, u] = pq.top(); pq.pop();
        if (du != d[u]) continue;
        for (auto [v, t] : g[u])
            if (du + t < d[v]) { d[v] = du + t; pq.push({d[v], v}); }
    }
    string out;
    for (int i = 1; i <= n; i++) out += to_string(d[i] == INF ? -1 : d[i]) + "\n";
    fputs(out.c_str(), stdout);
}
