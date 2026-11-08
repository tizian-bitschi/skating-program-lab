# Adult conditional-bound pruning (#130)

Issue: [#130](https://github.com/tizian-bitschi/skating-program-lab/issues/130).
Baseline: `40ea327651e8c771e8d8a3f5f7f22d6f34eb0333`.

The production change applies only after maximum proof to Adult requests with
no required occurrences. It reuses the existing admissible bound for one
hypothetically required exact atom. A bound strictly below the proved maximum
fixes that atom's count to zero. Equality remains permitted. The complete
[argument and cancellation contract](specs/planning-search.md#conditional-adult-bounds-after-maximum-proof-130)
are part of the planning-search specification.

The public API still returns at most five alternatives. Required-occurrence
requests and Senior retain the prior path. There are no environment switches,
alternative branchers or diagnostic logging in the production implementation.

## Regression evidence

`tests/planning_bound_pruning.rs` exercises the public search boundary:

- A caller-defined cooperative collection budget returns five Masters optima.
  Before the change, the same 14,000-check budget retains only four, with
  Cancelled status. After the change it reaches LimitReached. This is a broad
  work-budget regression, not an exact check-count or wall-clock assertion.
- A literal five-partition Bronze fixture checks all tied compound orders and
  the LimitReached meaning when exactly five optima exist.
- A two-partition fixture proves Exhausted; adding two exact required solos
  retains the established required-occurrence path and its single solution.
- Cancellation during Adult preprocessing preserves the proved representative.
- Personal single-jump limits retain literal source-derived Gold/Masters maxima.

`tests/planning_bound_pruning_oracle.rs` independently enumerates optimal
Gold/Masters jump partitions and nonjump choices from the public atomic grammar
and catalogue. Its resource and spin-role checks do not use Pumpkin or the
conditional-bound helper. It retains Euler budgets, internal compound order,
transition assumptions and repeated occurrences. The complete oracles contain
526 Gold and 244 Masters exact optima. The production tests verify that the five
returned identities are unique members of these sets, with correct BV, legal
assessment and LimitReached status. They do not bypass the public result cap.

During the preceding isolated investigation, an uncapped diagnostic copy also
matched those entire sets: Gold 526 jump partitions times one nonjump choice;
Masters 122 times two. Those complete-set experimental results motivate this
change; the production oracle regression deliberately retains the five-result
contract. Public grammar/catalogue reuse means this checks enumeration within
the supported repertoire rather than independently auditing all official rules.

Existing search, personal-limit, rotation, required-occurrence, Senior and
cancellation tests remain applicable. The implementation adds no new rule or
scoring interpretation.

## Identical-request Release measurements

Recorded 2026-11-08 on Windows, Intel Core i9-13900K, Rust 1.99.1, Pumpkin 0.5.0.
The existing `planning_readiness` example runs each empty Adult request in a
fresh process three times, serially, before and after the change. All requests
use default limits and same-direction preference. Compilation and stdin ACK
are outside API time. Request manifests match byte-for-byte across repetitions
and revisions; their SHA-256 values are included in the
[24 raw observations](../benchmarks/planning-readiness/observations-issue-130/default-five.csv).
Every run returns the literal maximum, five alternatives, LimitReached and a
valid shared assessment.

Median milliseconds:

| Profile | Proof before | Collection before | Total before | Proof after | Collection after | Total after |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | 8.210 | 1.305 | 9.502 | 8.682 | 1.260 | 9.942 |
| Silver | 15.293 | 2.229 | 18.318 | 16.325 | 2.598 | 19.068 |
| Gold | 27.080 | 6.148 | 33.590 | 26.895 | 5.665 | 32.502 |
| Masters | 44.903 | 1153.104 | 1198.007 | 48.021 | 14.568 | 62.155 |

Proof ends at MaximumProved, before the new preprocessing. Collection includes
preprocessing and subsequent alternative enumeration. Separate column medians
need not add to the total median. Masters total time improves by about 19 times;
Gold's first five were already cheap. Bronze/Silver differences are small
absolute changes with added preprocessing and uncontrolled host noise. These
observations establish no latency guarantee or improvement for every request.

The historical uncapped investigation took more than 300 seconds in the
baseline, versus roughly 0.49 s Gold and 0.33 s Masters after conditional pruning.
Those figures describe complete enumeration in a diagnostic copy, not the
production API's five-result behavior.

To reproduce one capped observation from either revision:

```powershell
cargo build --release --offline --example planning_readiness
'' | ./target/release/examples/planning_readiness.exe Masters empty
```

Repeat for Bronze, Silver and Gold. The existing benchmark README documents
the lifecycle columns and host limitations. CI uses functional and cooperative
budget assertions rather than wall-clock thresholds.
