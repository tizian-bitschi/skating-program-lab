# Native Release observations

Expectation edition: `issue-132`.

All 216 processes passed: 72 primary requests and 144 cancellation requests.
Three fresh-process repetitions per identical request; no watchdog failures.

Times below are median milliseconds. Range is minimum–maximum total time.
Memory is maximum lifetime peak working set across the three runs, in MiB.

| Profile | Case | BV | Domain | Model | First valid | Proof | Alternatives | Total | Total range | Peak MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | empty | 1100 | 8.449 | 0.111 | 8.857 | 9.127 | 2.241 | 11.369 | 11.136–12.236 | 5.70 |
| Bronze | required | 1100 | 8.126 | 0.114 | 8.585 | 8.858 | 3.090 | 11.949 | 11.638–12.433 | 5.69 |
| Bronze | excluded | 1100 | 8.611 | 0.128 | 9.106 | 9.394 | 2.352 | 11.919 | 11.636–12.018 | 5.60 |
| Bronze | impossible | Infeasible | — | — | — | — | — | 7.645 | 7.400–7.882 | 4.65 |
| Silver | empty | 1480 | 17.536 | 0.726 | 1195.619 | 1195.972 | 8.287 | 1204.234 | 1202.480–1215.573 | 17.45 |
| Silver | required | 1470 | 17.189 | 0.755 | 53.487 | 53.882 | 5.227 | 58.963 | 56.865–60.705 | 8.70 |
| Silver | excluded | 1480 | 17.702 | 0.554 | 1058.616 | 1059.006 | 6.814 | 1065.931 | 1056.934–1068.634 | 16.94 |
| Silver | impossible | Infeasible | — | — | — | — | — | 17.471 | 17.317–18.341 | 4.81 |
| Gold | empty | 2420 | 30.730 | 1.614 | 33.486 | 34.063 | 9.233 | 43.296 | 42.327–44.294 | 9.52 |
| Gold | required | 2420 | 30.309 | 1.587 | 34.122 | 34.696 | 7.080 | 41.576 | 40.877–43.417 | 10.75 |
| Gold | excluded | 2280 | 30.758 | 0.856 | 50.155 | 50.657 | 5.244 | 56.550 | 53.429–57.795 | 10.12 |
| Gold | impossible | Infeasible | — | — | — | — | — | 30.198 | 29.620–34.600 | 5.12 |
| Masters | empty | 3340 | 48.447 | 2.812 | 53.546 | 54.276 | 23.078 | 76.744 | 75.005–77.705 | 13.18 |
| Masters | required | 3280 | 46.496 | 3.782 | 53.682 | 54.433 | 607.587 | 662.021 | 648.913–670.968 | 54.61 |
| Masters | excluded | 3340 | 49.086 | 1.792 | 52.122 | 52.774 | 10.645 | 63.157 | 62.370–72.706 | 10.29 |
| Masters | impossible | Infeasible | — | — | — | — | — | 50.846 | 49.274–66.086 | 5.54 |
| Women | empty | 13310 | 230.300 | 20.918 | 301.304 | 303.899 | 94.690 | 398.590 | 397.750–404.818 | 46.58 |
| Women | required | 12020 | 230.884 | 21.442 | 324.355 | 327.084 | 121.251 | 450.213 | 448.335–453.087 | 51.72 |
| Women | excluded | 13310 | 228.132 | 10.833 | 243.433 | 245.272 | 66.120 | 311.393 | 310.584–311.612 | 33.80 |
| Women | impossible | Infeasible | — | — | — | — | — | 229.174 | 228.616–234.097 | 10.08 |
| Men | empty | 13310 | 234.071 | 21.874 | 308.249 | 311.526 | 93.649 | 405.176 | 398.737–414.883 | 46.46 |
| Men | required | 12020 | 233.505 | 21.424 | 326.519 | 329.346 | 118.872 | 450.098 | 443.895–451.597 | 52.02 |
| Men | excluded | 13310 | 231.263 | 11.767 | 248.319 | 250.199 | 68.952 | 317.172 | 316.600–328.982 | 33.69 |
| Men | impossible | Infeasible | — | — | — | — | — | 236.592 | 236.492–240.583 | 10.05 |

## Cancellation response

Trigger-to-return time in milliseconds over all profiles and repetitions.

| Trigger | Median | Maximum | Worst profile |
| --- | ---: | ---: | --- |
| Domain | 0.007 | 0.013 | Gold |
| Model | 0.151 | 0.935 | Women |
| Solve | 0.585 | 5.036 | Women |
| IncumbentFound | 2.273 | 7.184 | Men |
| Reconstruction | 2.186 | 6.890 | Women |
| MaximumProved | 2.304 | 8.459 | Women |
| Alternatives | 2.292 | 6.623 | Women |
| PostproofReconstruction | 2.550 | 8.239 | Men |
