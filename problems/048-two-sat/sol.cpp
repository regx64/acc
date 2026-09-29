#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m; scanf("%d %d", &n, &m);
    auto id = [&](int l) { return l > 0 ? 2 * (l - 1) : 2 * (-l - 1) + 1; };  // node for "l is true"
    int V = 2 * n;
    vector<vector<int>> g(V), r(V);
    for (int i = 0; i < m; i++) {
        int a, b; scanf("%d %d", &a, &b);
        // (a or b): not a -> b, not b -> a
        g[id(-a)].push_back(id(b)); g[id(-b)].push_back(id(a));
        r[id(b)].push_back(id(-a)); r[id(a)].push_back(id(-b));
    }
    vector<int> order; vector<char> seen(V, 0);
    for (int s = 0; s < V; s++) {
        if (seen[s]) continue;
        vector<pair<int, size_t>> st{{s, 0}}; seen[s] = 1;
        while (!st.empty()) {
            auto& [u, i] = st.back();
            if (i < g[u].size()) { int v = g[u][i++]; if (!seen[v]) { seen[v] = 1; st.push_back({v, 0}); } }
            else { order.push_back(u); st.pop_back(); }
        }
    }
    vector<int> comp(V, -1); int c = 0;
    for (int k = V - 1; k >= 0; k--) {
        int s = order[k]; if (comp[s] >= 0) continue;
        vector<int> st{s}; comp[s] = c;
        while (!st.empty()) { int u = st.back(); st.pop_back(); for (int v : r[u]) if (comp[v] < 0) { comp[v] = c; st.push_back(v); } }
        c++;
    }
    for (int i = 0; i < n; i++) if (comp[2 * i] == comp[2 * i + 1]) { puts("0"); return 0; }
    puts("1");
}
