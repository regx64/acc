#include <bits/stdc++.h>
using namespace std;
struct E { int to; long long cap; };
vector<E> es; vector<vector<int>> g; vector<int> lvl, it;
int n;
bool bfs(int s, int t) {
    lvl.assign(n + 1, -1); lvl[s] = 0; queue<int> q; q.push(s);
    while (!q.empty()) { int u = q.front(); q.pop();
        for (int id : g[u]) if (es[id].cap > 0 && lvl[es[id].to] < 0) { lvl[es[id].to] = lvl[u] + 1; q.push(es[id].to); } }
    return lvl[t] >= 0;
}
long long dfs(int u, int t, long long f) {
    if (u == t) return f;
    for (int& i = it[u]; i < (int)g[u].size(); i++) {
        int id = g[u][i]; E& e = es[id];
        if (e.cap > 0 && lvl[e.to] == lvl[u] + 1) {
            long long got = dfs(e.to, t, min(f, e.cap));
            if (got > 0) { e.cap -= got; es[id ^ 1].cap += got; return got; }
        }
    }
    return 0;
}
int main() {
    int m; scanf("%d %d", &n, &m);
    g.assign(n + 1, {});
    for (int i = 0; i < m; i++) {
        int u, v; long long c; scanf("%d %d %lld", &u, &v, &c);
        g[u].push_back(es.size()); es.push_back({v, c});
        g[v].push_back(es.size()); es.push_back({u, 0});
    }
    long long flow = 0;
    while (bfs(1, n)) { it.assign(n + 1, 0); while (long long f = dfs(1, n, LLONG_MAX)) flow += f; }
    printf("%lld\n", flow);
}
