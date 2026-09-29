#include <bits/stdc++.h>
using namespace std;
int main() {
    int n, m; scanf("%d %d", &n, &m);
    vector<vector<int>> g(n + 1), r(n + 1);
    for (int i = 0; i < m; i++) { int u, v; scanf("%d %d", &u, &v); g[u].push_back(v); r[v].push_back(u); }
    // Kosaraju, iterative.
    vector<int> order; order.reserve(n);
    vector<char> seen(n + 1, 0);
    vector<pair<int, size_t>> st;
    for (int s = 1; s <= n; s++) {
        if (seen[s]) continue;
        seen[s] = 1; st.push_back({s, 0});
        while (!st.empty()) {
            auto& [u, i] = st.back();
            if (i < g[u].size()) {
                int v = g[u][i++];
                if (!seen[v]) { seen[v] = 1; st.push_back({v, 0}); }
            } else { order.push_back(u); st.pop_back(); }
        }
    }
    vector<int> comp(n + 1, -1);
    int count = 0, best = 0;
    for (int k = n - 1; k >= 0; k--) {
        int s = order[k];
        if (comp[s] >= 0) continue;
        int size = 0; vector<int> stack = {s}; comp[s] = count;
        while (!stack.empty()) {
            int u = stack.back(); stack.pop_back(); size++;
            for (int v : r[u]) if (comp[v] < 0) { comp[v] = count; stack.push_back(v); }
        }
        best = max(best, size); count++;
    }
    printf("%d %d\n", count, best);
}
