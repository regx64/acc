#include <bits/stdc++.h>
using namespace std;
int n; vector<int> t;
void upd(int i, int x) { for (t[i += n] = x; i > 1; i >>= 1) t[i >> 1] = min(t[i], t[i ^ 1]); }
int qry(int l, int r) { // [l, r)
    int res = INT_MAX;
    for (l += n, r += n; l < r; l >>= 1, r >>= 1) {
        if (l & 1) res = min(res, t[l++]);
        if (r & 1) res = min(res, t[--r]);
    }
    return res;
}
int main() {
    int q; scanf("%d %d", &n, &q);
    t.assign(2 * n, INT_MAX);
    for (int i = 0; i < n; i++) scanf("%d", &t[n + i]);
    for (int i = n - 1; i >= 1; i--) t[i] = min(t[2 * i], t[2 * i + 1]);
    string out;
    while (q--) {
        int c, a, b; scanf("%d %d %d", &c, &a, &b);
        if (c == 1) upd(a - 1, b);
        else out += to_string(qry(a - 1, b)) + "\n";
    }
    fputs(out.c_str(), stdout);
}
