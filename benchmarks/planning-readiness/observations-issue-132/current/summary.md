# Native Release observations

Expectation edition: `issue-132`.

All 216 processes passed: 72 primary requests and 144 cancellation requests.
Three fresh-process repetitions per identical request; no watchdog failures.

Times below are median milliseconds. Range is minimum–maximum total time.
Memory is maximum lifetime peak working set across the three runs, in MiB.

| Profile | Case | BV | Domain | Model | First valid | Proof | Alternatives | Total | Total range | Peak MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100 | 8.628 | 0.115 | 9.032 | 9.265 | 2.492 | 11.737 | 11.330–12.191 | 6.06 |
| Bronze | required | 1100 | 8.109 | 0.120 | 8.543 | 8.794 | 3.459 | 12.165 | 12.087–12.320 | 5.65 |
| Bronze | excluded | 1100 | 8.629 | 0.119 | 9.154 | 9.431 | 2.653 | 11.968 | 11.835–12.175 | 5.64 |
| Bronze | impossible | Infeasible | — | — | — | — | — | 7.997 | 7.928–8.021 | 4.66 |
| Silver | empty | 1480 | 17.823 | 0.744 | 1182.456 | 1182.805 | 8.367 | 1191.156 | 1177.871–1196.571 | 17.38 |
| Silver | required | 1470 | 16.929 | 0.803 | 52.841 | 53.250 | 4.984 | 58.162 | 57.738–61.064 | 8.75 |
| Silver | excluded | 1480 | 17.680 | 0.525 | 1053.519 | 1053.842 | 6.523 | 1060.045 | 1056.963–1075.664 | 17.23 |
| Silver | impossible | Infeasible | — | — | — | — | — | 17.369 | 17.349–18.077 | 4.82 |
| Gold | empty | 2420 | 29.499 | 1.675 | 32.570 | 33.202 | 8.758 | 41.960 | 41.052–52.111 | 9.53 |
| Gold | required | 2420 | 28.945 | 1.547 | 32.535 | 33.091 | 7.072 | 40.306 | 40.163–43.776 | 10.56 |
| Gold | excluded | 2280 | 30.800 | 0.922 | 49.577 | 50.060 | 5.703 | 55.763 | 54.968–55.863 | 10.09 |
| Gold | impossible | Infeasible | — | — | — | — | — | 30.427 | 30.276–30.701 | 5.11 |
| Masters | empty | 3340 | 49.013 | 2.814 | 54.324 | 55.105 | 21.647 | 76.752 | 74.908–77.183 | 13.22 |
| Masters | required | 3280 | 46.573 | 2.885 | 54.265 | 55.089 | 617.016 | 672.105 | 664.500–683.107 | 54.60 |
| Masters | excluded | 3340 | 49.193 | 1.756 | 51.965 | 52.723 | 10.571 | 63.109 | 62.827–67.139 | 10.17 |
| Masters | impossible | Infeasible | — | — | — | — | — | 50.104 | 48.375–51.092 | 5.57 |
| Women | empty | 13310 | 236.352 | 22.641 | 306.670 | 309.473 | 99.239 | 407.321 | 403.257–459.687 | 46.53 |
| Women | required | 12020 | 231.947 | 21.891 | 325.441 | 328.197 | 120.726 | 447.206 | 445.759–454.796 | 52.54 |
| Women | excluded | 13310 | 230.099 | 13.141 | 247.162 | 249.034 | 67.865 | 315.953 | 315.762–318.784 | 33.62 |
| Women | impossible | Infeasible | — | — | — | — | — | 229.962 | 228.603–232.519 | 10.04 |
| Men | empty | 13310 | 233.831 | 23.814 | 315.629 | 318.511 | 95.460 | 411.934 | 403.611–435.160 | 46.34 |
| Men | required | 12020 | 240.003 | 22.730 | 333.711 | 337.012 | 129.443 | 466.455 | 459.727–483.330 | 51.96 |
| Men | excluded | 13310 | 237.371 | 11.544 | 254.948 | 257.041 | 68.597 | 325.639 | 324.583–331.160 | 33.72 |
| Men | impossible | Infeasible | — | — | — | — | — | 232.693 | 230.288–234.836 | 10.04 |

## Cancellation response

Trigger-to-return time in milliseconds over all profiles and repetitions.

| Trigger | Median | Maximum | Worst profile |
| --- | ---: | ---: | --- |
| Domain | 0.008 | 0.014 | Men |
| Model | 0.137 | 1.009 | Men |
| Solve | 0.560 | 5.361 | Men |
| IncumbentFound | 2.279 | 6.003 | Women |
| Reconstruction | 2.162 | 7.973 | Men |
| MaximumProved | 2.247 | 8.682 | Women |
| Alternatives | 2.207 | 5.972 | Women |
| PostproofReconstruction | 2.380 | 8.248 | Women |
