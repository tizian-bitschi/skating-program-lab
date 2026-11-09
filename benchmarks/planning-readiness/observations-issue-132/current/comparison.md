# Identical-request native Release comparison

Three serial fresh processes per case. Times are medians in milliseconds; ratio is current/baseline.
Proof includes validated representative reconstruction. Alternatives are measured separately.

| Profile | Case | BV before→after | Proof before→after | Ratio | Alternatives before→after | Ratio |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100→1100 | 8.627→9.127 | 1.06× | 1.134→2.241 | 1.98× |
| Bronze | required | 1100→1100 | 8.979→8.858 | 0.99× | 1.926→3.090 | 1.60× |
| Bronze | excluded | 1100→1100 | 8.756→9.394 | 1.07× | 1.106→2.352 | 2.13× |
| Bronze | impossible | Infeasible→Infeasible | — | — | — | — |
| Silver | empty | 1430→1480 | 16.019→1195.972 | 74.66× | 2.621→8.287 | 3.16× |
| Silver | required | 1420→1470 | 16.071→53.882 | 3.35× | 2.517→5.227 | 2.08× |
| Silver | excluded | 1430→1480 | 15.295→1059.006 | 69.24× | 2.192→6.814 | 3.11× |
| Silver | impossible | Infeasible→Infeasible | — | — | — | — |
| Gold | empty | 2420→2420 | 26.369→34.063 | 1.29× | 5.696→9.233 | 1.62× |
| Gold | required | 2420→2420 | 28.718→34.696 | 1.21× | 4.892→7.080 | 1.45× |
| Gold | excluded | 2280→2280 | 36.303→50.657 | 1.40× | 3.359→5.244 | 1.56× |
| Gold | impossible | Infeasible→Infeasible | — | — | — | — |
| Masters | empty | 3340→3340 | 41.473→54.276 | 1.31× | 13.372→23.078 | 1.73× |
| Masters | required | 3280→3280 | 43.261→54.433 | 1.26× | 570.115→607.587 | 1.07× |
| Masters | excluded | 3340→3340 | 41.122→52.774 | 1.28× | 6.314→10.645 | 1.69× |
| Masters | impossible | Infeasible→Infeasible | — | — | — | — |
| Women | empty | 13310→13310 | 261.876→303.899 | 1.16× | 89.355→94.690 | 1.06× |
| Women | required | 12020→12020 | 268.365→327.084 | 1.22× | 78.200→121.251 | 1.55× |
| Women | excluded | 13310→13310 | 215.374→245.272 | 1.14× | 62.239→66.120 | 1.06× |
| Women | impossible | Infeasible→Infeasible | — | — | — | — |
| Men | empty | 13310→13310 | 262.531→311.526 | 1.19× | 87.334→93.649 | 1.07× |
| Men | required | 12020→12020 | 277.387→329.346 | 1.19× | 78.936→118.872 | 1.51× |
| Men | excluded | 13310→13310 | 221.310→250.199 | 1.13× | 64.355→68.952 | 1.07× |
| Men | impossible | Infeasible→Infeasible | — | — | — | — |
