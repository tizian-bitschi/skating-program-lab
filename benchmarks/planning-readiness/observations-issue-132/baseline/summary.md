# Native Release observations

Expectation edition: `baseline-caa4127`.

All 216 processes passed: 72 primary requests and 144 cancellation requests.
Three fresh-process repetitions per identical request; no watchdog failures.

Times below are median milliseconds. Range is minimum–maximum total time.
Memory is maximum lifetime peak working set across the three runs, in MiB.

| Profile | Case | BV | Domain | Model | First valid | Proof | Alternatives | Total | Total range | Peak MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100 | 7.917 | 0.128 | 8.361 | 8.627 | 1.134 | 9.761 | 9.738–10.889 | 5.82 |
| Bronze | required | 1100 | 8.348 | 0.119 | 8.749 | 8.979 | 1.926 | 11.197 | 10.888–11.683 | 5.52 |
| Bronze | excluded | 1100 | 8.052 | 0.100 | 8.470 | 8.756 | 1.106 | 9.838 | 9.096–10.015 | 5.49 |
| Bronze | impossible | Infeasible | — | — | — | — | — | 7.618 | 7.508–8.045 | 4.59 |
| Silver | empty | 1430 | 14.730 | 0.454 | 15.689 | 16.019 | 2.621 | 18.610 | 18.147–25.396 | 6.51 |
| Silver | required | 1420 | 14.655 | 0.492 | 15.674 | 16.071 | 2.517 | 18.645 | 17.532–19.077 | 6.55 |
| Silver | excluded | 1430 | 14.191 | 0.330 | 14.966 | 15.295 | 2.192 | 17.487 | 17.123–19.072 | 6.00 |
| Silver | impossible | Infeasible | — | — | — | — | — | 14.976 | 13.871–15.537 | 4.70 |
| Gold | empty | 2420 | 23.884 | 0.967 | 25.885 | 26.369 | 5.696 | 32.126 | 30.384–33.203 | 7.90 |
| Gold | required | 2420 | 25.859 | 0.983 | 28.169 | 28.718 | 4.892 | 33.544 | 32.022–33.725 | 8.23 |
| Gold | excluded | 2280 | 23.244 | 0.555 | 35.816 | 36.303 | 3.359 | 39.662 | 38.075–40.682 | 7.77 |
| Gold | impossible | Infeasible | — | — | — | — | — | 24.930 | 22.672–26.023 | 4.90 |
| Masters | empty | 3340 | 36.804 | 1.837 | 40.845 | 41.473 | 13.372 | 55.214 | 54.008–55.619 | 10.45 |
| Masters | required | 3280 | 38.045 | 1.957 | 42.421 | 43.261 | 570.115 | 614.833 | 605.715–621.403 | 40.09 |
| Masters | excluded | 3340 | 38.154 | 1.124 | 40.516 | 41.122 | 6.314 | 47.436 | 46.249–48.781 | 8.32 |
| Masters | impossible | Infeasible | — | — | — | — | — | 38.886 | 38.034–42.005 | 5.20 |
| Women | empty | 13310 | 200.498 | 18.526 | 259.539 | 261.876 | 89.355 | 351.232 | 336.525–380.223 | 37.10 |
| Women | required | 12020 | 200.485 | 18.822 | 266.139 | 268.365 | 78.200 | 346.565 | 336.218–379.459 | 37.16 |
| Women | excluded | 13310 | 200.967 | 8.828 | 213.688 | 215.374 | 62.239 | 277.614 | 266.796–297.961 | 27.37 |
| Women | impossible | Infeasible | — | — | — | — | — | 200.991 | 195.796–210.625 | 9.02 |
| Men | empty | 13310 | 201.195 | 18.296 | 260.147 | 262.531 | 87.334 | 353.793 | 341.194–356.457 | 37.33 |
| Men | required | 12020 | 207.946 | 18.480 | 274.942 | 277.387 | 78.936 | 355.347 | 344.285–382.672 | 37.50 |
| Men | excluded | 13310 | 204.789 | 8.941 | 218.612 | 221.310 | 64.355 | 285.665 | 263.756–296.360 | 27.49 |
| Men | impossible | Infeasible | — | — | — | — | — | 199.250 | 192.343–201.854 | 9.05 |

## Cancellation response

Trigger-to-return time in milliseconds over all profiles and repetitions.

| Trigger | Median | Maximum | Worst profile |
| --- | ---: | ---: | --- |
| Domain | 0.008 | 0.009 | Masters |
| Model | 0.100 | 0.828 | Men |
| Solve | 0.390 | 4.223 | Women |
| IncumbentFound | 0.458 | 4.776 | Women |
| Reconstruction | 0.478 | 6.309 | Men |
| MaximumProved | 0.448 | 4.770 | Men |
| Alternatives | 0.465 | 7.246 | Men |
| PostproofReconstruction | 0.558 | 6.360 | Women |
