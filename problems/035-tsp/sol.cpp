#include <bits/stdc++.h>
using namespace std;
int main() {
    int n; scanf("%d", &n);
    vector<vector<long long>> c(n, vector<long long>(n));
    for (auto& r : c) for (auto& x : r) scanf("%lld", &x);
    const long long INF = LLONG_MAX / 4;
    vector<vector<long long>> dp(1 << n, vector<long long>(n, INF));
    dp[1][0] = 0;
    for (int m = 1; m < (1 << n); m++)
        for (int u = 0; u < n; u++) {
            if (dp[m][u] >= INF || !(m >> u & 1)) continue;
            for (int v = 0; v < n; v++)
                if (!(m >> v & 1) && c[u][v] > 0)
                    dp[m | 1 << v][v] = min(dp[m | 1 << v][v], dp[m][u] + c[u][v]);
        }
    long long best = INF;
    for (int u = 1; u < n; u++)
        if (dp[(1 << n) - 1][u] < INF && c[u][0] > 0) best = min(best, dp[(1 << n) - 1][u] + c[u][0]);
    printf("%lld\n", best >= INF ? -1 : best);
}
