# Personal limits verification (#128)

Base: `29fe7a69fff76e2903d097b9752610f125172dfb` (`origin/main`).
Toolchain: Rust 1.99.1 (`58cf8f9c3`, 2026-10-13), Cargo 1.99.1
(`5f94df478`), LLVM 23.1.1, `x86_64-pc-windows-msvc`, Pumpkin 0.5.0.

## Checks

All required checks passed:

```text
cargo build --offline
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
cargo test --offline --no-default-features
```

The final default-feature inventory contains 402 tests: 400 passed across the
full suite and final focused regressions; two existing wall-clock diagnostic
benchmarks remain ignored. No-default-features passed all 264 tests. The final
focused runs include all 12 personal-limit tests, all eight finite jump tests,
all 22 completion tests and the two personal mixed-optimizer regressions.
No numerical historical fixture changed.

The full default suite completed successfully in the implementation tool
transcript. Its raw stdout was not retained as a file. Additional logs live in
`target/issue-128-verification/` in the implementation worktree:

- `toolchain.txt`, `build.txt`, `fmt.txt`, `clippy.txt`
- `default-test-list.txt` (final inventory, not an execution log)
- `no-default-tests.txt` (complete no-default execution)
- `personal-regressions.txt`, `finite-regressions.txt`
- `release-smoke.txt` (complete requests and native Release measurements)

These are local verification artifacts, excluded by the repository's existing
`target/` rule. The focused commands were:

```text
cargo test --offline --test personal_limits --test jump_optimizer --test program_completion
cargo test --offline --test program_optimizer personal_
```

Cargo initially could not resolve the crates.io hostname. All checks used the
available offline cache. Initial compile failures confirmed the missing public
APIs before their implementation. One fixture incorrectly expected CoSp under
Adult Gold; the test now uses Senior, where that abbreviation is selectable.
The worked Senior bound was corrected to reserve the mandatory ChSp spin slot.
No production fallback or changed scoring fixture was used to resolve failures.
Incremental hard-link warnings on this filesystem were avoided in final checks
with `CARGO_INCREMENTAL=0`.

## Native Release smoke

The request-shape migration changes benchmark Debug manifests, but empty limits
do not change their effective domains or expected values. Historical manifests
remain tied to their original revision. A serial native smoke run rebuilt the
existing runner and exercised all six empty profile requests with no concurrent
compiler or test job:

```text
cargo build --offline --release --example planning_readiness
```

For each of `Bronze Silver Gold Masters Women Men`, an empty line was piped to
`target/release/examples/planning_readiness.exe <profile> empty` to acknowledge
the runner's final measurement. Every process exited successfully, proved its
literal maximum and returned five alternatives with `LimitReached`.

| Profile | Maximum BV | Total ms |
| --- | ---: | ---: |
| Bronze | 11.00 | 10.882 |
| Silver | 14.30 | 18.385 |
| Gold | 24.20 | 33.570 |
| Masters | 33.40 | 1207.272 |
| Senior Women | 133.10 | 350.200 |
| Senior Men | 133.10 | 361.268 |

This is one smoke observation per request, not a repeated performance comparison
or a latency guarantee. The 216-run historical experiment and the two ignored
wall-clock diagnostic benchmarks were not repeated. The new limited-domain
integration tests independently verify Bronze 10.20 and both Senior maxima
21.70 with Basic-spin and Single-jump maxima, plus impossible-repertoire,
required-conflict, manual-assessment and immutable-provenance behavior.
