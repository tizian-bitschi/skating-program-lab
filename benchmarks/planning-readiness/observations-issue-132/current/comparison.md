# Identical-request native Release comparison

Three serial fresh processes per case. Times are medians in milliseconds; ratio is current/baseline.
Proof includes validated representative reconstruction. Alternatives are measured separately.

| Profile | Case | BV before→after | Proof before→after | Ratio | Alternatives before→after | Ratio |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100→1100 | 8.627→9.265 | 1.07× | 1.134→2.492 | 2.20× |
| Bronze | required | 1100→1100 | 8.979→8.794 | 0.98× | 1.926→3.459 | 1.80× |
| Bronze | excluded | 1100→1100 | 8.756→9.431 | 1.08× | 1.106→2.653 | 2.40× |
| Bronze | impossible | Infeasible→Infeasible | — | — | — | — |
| Silver | empty | 1430→1480 | 16.019→1182.805 | 73.84× | 2.621→8.367 | 3.19× |
| Silver | required | 1420→1470 | 16.071→53.250 | 3.31× | 2.517→4.984 | 1.98× |
| Silver | excluded | 1430→1480 | 15.295→1053.842 | 68.90× | 2.192→6.523 | 2.98× |
| Silver | impossible | Infeasible→Infeasible | — | — | — | — |
| Gold | empty | 2420→2420 | 26.369→33.202 | 1.26× | 5.696→8.758 | 1.54× |
| Gold | required | 2420→2420 | 28.718→33.091 | 1.15× | 4.892→7.072 | 1.45× |
| Gold | excluded | 2280→2280 | 36.303→50.060 | 1.38× | 3.359→5.703 | 1.70× |
| Gold | impossible | Infeasible→Infeasible | — | — | — | — |
| Masters | empty | 3340→3340 | 41.473→55.105 | 1.33× | 13.372→21.647 | 1.62× |
| Masters | required | 3280→3280 | 43.261→55.089 | 1.27× | 570.115→617.016 | 1.08× |
| Masters | excluded | 3340→3340 | 41.122→52.723 | 1.28× | 6.314→10.571 | 1.67× |
| Masters | impossible | Infeasible→Infeasible | — | — | — | — |
| Women | empty | 13310→13310 | 261.876→309.473 | 1.18× | 89.355→99.239 | 1.11× |
| Women | required | 12020→12020 | 268.365→328.197 | 1.22× | 78.200→120.726 | 1.54× |
| Women | excluded | 13310→13310 | 215.374→249.034 | 1.16× | 62.239→67.865 | 1.09× |
| Women | impossible | Infeasible→Infeasible | — | — | — | — |
| Men | empty | 13310→13310 | 262.531→318.511 | 1.21× | 87.334→95.460 | 1.09× |
| Men | required | 12020→12020 | 277.387→337.012 | 1.21× | 78.936→129.443 | 1.64× |
| Men | excluded | 13310→13310 | 221.310→257.041 | 1.16× | 64.355→68.597 | 1.07× |
| Men | impossible | Infeasible→Infeasible | — | — | — | — |
