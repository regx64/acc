#include <bits/stdc++.h>
using namespace std;
// Hopcroft-Karp
int n, m;
vector<vector<int>> g;
vector<int> mu, mv, dist;
bool bfs() {
    queue<int> q; bool found = false;
    for (int u = 1; u <= n; u++) { if (!mu[u]) { dist[u] = 0; q.push(u); } else dist[u] = -1; }
    while (!q.empty()) {
        int u = q.front(); q.pop();
        for (int v : g[u]) {
            int w = mv[v];
            if (!w) found = true;
            else if (dist[w] < 0) { dist[w] = dist[u] + 1; q.push(w); }
        }
    }
    return found;
}
bool dfs(int u) {
    for (int v : g[u]) {
        int w = mv[v];
        if (!w || (dist[w] == dist[u] + 1 && dfs(w))) { mu[u] = v; mv[v] = u; return true; }
    }
    dist[u] = -1;
    return false;
}
int main() {
    int e; scanf("%d %d %d", &n, &m, &e);
    g.assign(n + 1, {}); mu.assign(n + 1, 0); mv.assign(m + 1, 0); dist.assign(n + 1, 0);
    for (int i = 0; i < e; i++) { int a, b; scanf("%d %d", &a, &b); g[a].push_back(b); }
    int res = 0;
    while (bfs()) for (int u = 1; u <= n; u++) if (!mu[u] && dfs(u)) res++;
    printf("%d\n", res);
}
