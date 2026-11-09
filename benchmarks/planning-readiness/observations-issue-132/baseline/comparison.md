# Identical-request native Release comparison

Three serial fresh processes per case. Times are medians in milliseconds; ratio is current/baseline.
Proof includes validated representative reconstruction. Alternatives are measured separately.

| Profile | Case | BV before→after | Proof before→after | Ratio | Alternatives before→after | Ratio |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100→1100 | 8.627→8.627 | 1.00× | 1.134→1.134 | 1.00× |
| Bronze | required | 1100→1100 | 8.979→8.979 | 1.00× | 1.926→1.926 | 1.00× |
| Bronze | excluded | 1100→1100 | 8.756→8.756 | 1.00× | 1.106→1.106 | 1.00× |
| Bronze | impossible | Infeasible→Infeasible | — | — | — | — |
| Silver | empty | 1430→1430 | 16.019→16.019 | 1.00× | 2.621→2.621 | 1.00× |
| Silver | required | 1420→1420 | 16.071→16.071 | 1.00× | 2.517→2.517 | 1.00× |
| Silver | excluded | 1430→1430 | 15.295→15.295 | 1.00× | 2.192→2.192 | 1.00× |
| Silver | impossible | Infeasible→Infeasible | — | — | — | — |
| Gold | empty | 2420→2420 | 26.369→26.369 | 1.00× | 5.696→5.696 | 1.00× |
| Gold | required | 2420→2420 | 28.718→28.718 | 1.00× | 4.892→4.892 | 1.00× |
| Gold | excluded | 2280→2280 | 36.303→36.303 | 1.00× | 3.359→3.359 | 1.00× |
| Gold | impossible | Infeasible→Infeasible | — | — | — | — |
| Masters | empty | 3340→3340 | 41.473→41.473 | 1.00× | 13.372→13.372 | 1.00× |
| Masters | required | 3280→3280 | 43.261→43.261 | 1.00× | 570.115→570.115 | 1.00× |
| Masters | excluded | 3340→3340 | 41.122→41.122 | 1.00× | 6.314→6.314 | 1.00× |
| Masters | impossible | Infeasible→Infeasible | — | — | — | — |
| Women | empty | 13310→13310 | 261.876→261.876 | 1.00× | 89.355→89.355 | 1.00× |
| Women | required | 12020→12020 | 268.365→268.365 | 1.00× | 78.200→78.200 | 1.00× |
| Women | excluded | 13310→13310 | 215.374→215.374 | 1.00× | 62.239→62.239 | 1.00× |
| Women | impossible | Infeasible→Infeasible | — | — | — | — |
| Men | empty | 13310→13310 | 262.531→262.531 | 1.00× | 87.334→87.334 | 1.00× |
| Men | required | 12020→12020 | 277.387→277.387 | 1.00× | 78.936→78.936 | 1.00× |
| Men | excluded | 13310→13310 | 221.310→221.310 | 1.00× | 64.355→64.355 | 1.00× |
| Men | impossible | Infeasible→Infeasible | — | — | — | — |
