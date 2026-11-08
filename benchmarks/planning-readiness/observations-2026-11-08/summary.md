# Native Release observations

All 216 processes passed: 72 primary requests and 144 cancellation requests.
Three fresh-process repetitions per identical request; no watchdog failures.

Times below are median milliseconds. Range is minimum–maximum total time.
Memory is maximum lifetime peak working set across the three runs, in MiB.

| Profile | Case | BV | Domain | Model | First valid | Proof | Alternatives | Total | Total range | Peak MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100 | 7.618 | 0.151 | 8.115 | 8.339 | 1.315 | 9.654 | 8.993–9.955 | 5.98 |
| Bronze | required | 1100 | 7.764 | 0.133 | 8.200 | 8.442 | 1.378 | 9.892 | 9.712–10.026 | 5.45 |
| Bronze | excluded | 1100 | 7.606 | 0.121 | 8.112 | 8.335 | 1.215 | 9.550 | 9.189–10.250 | 5.35 |
| Bronze | impossible | Infeasible | — | — | — | — | — | 7.696 | 7.534–7.894 | 4.54 |
| Silver | empty | 1430 | 14.243 | 1.306 | 17.069 | 17.565 | 1.826 | 19.337 | 19.171–19.778 | 7.91 |
| Silver | required | 1420 | 14.190 | 1.309 | 16.543 | 17.059 | 2.053 | 19.112 | 18.841–19.238 | 7.78 |
| Silver | excluded | 1430 | 14.228 | 0.867 | 15.904 | 16.292 | 1.561 | 17.854 | 17.536–17.901 | 6.96 |
| Silver | impossible | Infeasible | — | — | — | — | — | 13.846 | 13.835–14.260 | 4.79 |
| Gold | empty | 2420 | 22.142 | 3.213 | 35.102 | 35.851 | 3.410 | 39.262 | 38.387–40.802 | 12.72 |
| Gold | required | 2420 | 23.236 | 3.429 | 37.549 | 38.387 | 3.462 | 41.849 | 39.040–43.243 | 12.96 |
| Gold | excluded | 2280 | 23.306 | 1.845 | 28.014 | 28.636 | 2.736 | 31.372 | 30.340–31.677 | 9.56 |
| Gold | impossible | Infeasible | — | — | — | — | — | 23.045 | 22.262–23.296 | 5.42 |
| Masters | empty | 3340 | 34.768 | 8.691 | 102.220 | 104.032 | 6.496 | 110.720 | 103.285–111.369 | 23.38 |
| Masters | required | 3280 | 34.402 | 8.878 | 101.191 | 102.761 | 6.111 | 108.851 | 105.582–110.577 | 23.11 |
| Masters | excluded | 3340 | 34.575 | 5.378 | 59.116 | 60.429 | 4.748 | 65.080 | 64.936–69.048 | 16.92 |
| Masters | impossible | Infeasible | — | — | — | — | — | 35.101 | 34.257–36.553 | 6.71 |
| Women | empty | 13310 | 171.174 | 103.663 | 2965.587 | 2975.370 | 71.195 | 3046.566 | 2774.865–3282.770 | 150.26 |
| Women | required | 12020 | 164.206 | 100.959 | 2946.231 | 2956.866 | 47.312 | 3004.178 | 2948.543–3097.989 | 150.62 |
| Women | excluded | 13310 | 169.009 | 59.012 | 1015.796 | 1021.728 | 23.266 | 1044.994 | 1002.850–1081.371 | 96.22 |
| Women | impossible | Infeasible | — | — | — | — | — | 165.029 | 162.703–168.878 | 20.59 |
| Men | empty | 13310 | 172.014 | 118.283 | 2851.455 | 2860.744 | 37.217 | 2897.961 | 2816.828–3015.176 | 150.54 |
| Men | required | 12020 | 171.458 | 105.697 | 2896.361 | 2906.085 | 38.986 | 2942.759 | 2914.588–3277.661 | 150.69 |
| Men | excluded | 13310 | 170.724 | 63.242 | 1017.792 | 1024.122 | 26.685 | 1048.846 | 1037.588–1073.248 | 96.23 |
| Men | impossible | Infeasible | — | — | — | — | — | 174.342 | 172.491–177.460 | 20.55 |

## Cancellation response

Trigger-to-return time in milliseconds over all profiles and repetitions.

| Trigger | Median | Maximum | Worst profile |
| --- | ---: | ---: | --- |
| Domain | 0.005 | 0.006 | Bronze |
| Model | 0.322 | 3.224 | Men |
| Solve | 1.561 | 23.493 | Women |
| IncumbentFound | 1.967 | 45.506 | Men |
| Reconstruction | 1.966 | 58.640 | Men |
| MaximumProved | 1.798 | 63.544 | Men |
| Alternatives | 1.866 | 74.925 | Men |
| PostproofReconstruction | 1.910 | 94.142 | Men |
