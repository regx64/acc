#include <bits/stdc++.h>
using namespace std;
// Suffix automaton: answer = sum over states of len(v) - len(link(v)).
struct St { int len, link; int next[26]; };
int main() {
    static char buf[200005];
    if (scanf("%200004s", buf) != 1) return 0;
    string s = buf;
    vector<St> st; st.reserve(2 * s.size() + 2);
    St root{}; root.len = 0; root.link = -1; memset(root.next, -1, sizeof root.next);
    st.push_back(root);
    int last = 0;
    for (char ch : s) {
        int c = ch - 'a';
        int cur = st.size();
        St ns{}; ns.len = st[last].len + 1; ns.link = 0; memset(ns.next, -1, sizeof ns.next);
        st.push_back(ns);
        int p = last;
        while (p != -1 && st[p].next[c] == -1) { st[p].next[c] = cur; p = st[p].link; }
        if (p != -1) {
            int q = st[p].next[c];
            if (st[p].len + 1 == st[q].len) st[cur].link = q;
            else {
                int clone = st.size();
                St cl = st[q]; cl.len = st[p].len + 1;
                st.push_back(cl);
                while (p != -1 && st[p].next[c] == q) { st[p].next[c] = clone; p = st[p].link; }
                st[q].link = st[cur].link = clone;
            }
        }
        last = cur;
    }
    long long ans = 0;
    for (size_t v = 1; v < st.size(); v++) ans += st[v].len - st[st[v].link].len;
    printf("%lld\n", ans);
}
