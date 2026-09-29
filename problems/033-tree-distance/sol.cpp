#include <bits/stdc++.h>
using namespace std;
int main() {
    int n; scanf("%d", &n);
    const int LOG = 17;
    vector<array<int, LOG + 1>> up(n + 1);
    vector<vector<int>> ch(n + 1);
    for (int v = 2; v <= n; v++) { int p; scanf("%d", &p); ch[p].push_back(v); up[v][0] = p; }
    up[1][0] = 1;
    vector<int> depth(n + 1, 0), order; order.reserve(n);
    order.push_back(1);
    for (size_t i = 0; i < order.size(); i++)
        for (int c : ch[order[i]]) { depth[c] = depth[order[i]] + 1; order.push_back(c); }
    for (int k = 1; k <= LOG; k++)
        for (int v : order) up[v][k] = up[up[v][k - 1]][k - 1];
    int q; scanf("%d", &q);
    string out;
    while (q--) {
        int a, b; scanf("%d %d", &a, &b);
        int x = a, y = b;
        if (depth[x] < depth[y]) swap(x, y);
        for (int k = LOG; k >= 0; k--) if (depth[x] - (1 << k) >= depth[y]) x = up[x][k];
        if (x != y) {
            for (int k = LOG; k >= 0; k--) if (up[x][k] != up[y][k]) { x = up[x][k]; y = up[y][k]; }
            x = up[x][0];
        }
        out += to_string(depth[a] + depth[b] - 2 * depth[x]) + " " + to_string(x) + "\n";
    }
    fputs(out.c_str(), stdout);
}
