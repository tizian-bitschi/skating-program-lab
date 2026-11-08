# Rotation preference validation

Date: 2026-11-08. Implementation branch: `codex/rotation-preference`.
Contract: [rotation preference](specs/rotation-preference-proposal.md).

The production diff excludes the HTML prototype. The prototype remains on
`codex/prototype-rotation-preference` and in the separate `rotation-preference-demo`
worktree.

## Behavioral evidence

- Full six-profile code-set equality against independent connector productions
  for the standard setting, plus the existing independent broad grammar for opt-in.
- Seven public-API regression tests cover full-element filtering, the original
  manual 14.30 program without altered rule findings, detailed manual separation,
  Silver proof in both modes, required-input conflicts, optional manual candidates,
  finite optimization, immutable originals with both lock flags and generated-domain
  preference mismatch.
- Cancellation during the new filtering phase exposes no partial repertoire.
- The small Bronze exhaustive oracle verifies both settings: three standard
  maxima are exhausted, while five broad maxima reach the alternative limit.
- The six-profile required-occurrence/exclusion scenario runs with the standard
  setting. Its Masters case required a tighter admissible upper bound; the
  documented component-cap relaxation removes unattainable objective values.

## Native Debug smoke runs

Each run used `cargo run --offline --example planning_readiness -- <profile> empty`
with a newline on stdin for the harness acknowledgement. Requests explicitly
contain `SameDirection`, no requirements and no exclusions.

| Profile | Proven BV | Returned alternatives | Alternative status |
| --- | ---: | ---: | --- |
| Bronze | 11.00 | 5 | LimitReached |
| Silver | 14.30 | 5 | LimitReached |
| Gold | 24.20 | 5 | LimitReached |
| Masters | 33.40 | 5 | LimitReached |
| Senior Women | 133.10 | 5 | LimitReached |
| Senior Men | 133.10 | 5 | LimitReached |

Every harness check passed. These are Debug correctness smoke runs, conducted
alongside tests, not controlled Release performance comparisons. LimitReached
does not claim that all equal optima have been enumerated. Proof covers the
selected theoretical domain and supported rules, not personal skating ability.

## Required checks

Passed with locally cached dependencies (`--offline` for Cargo dependency access):

- `cargo fmt --check`
- `cargo clippy --offline --all-targets -- -D warnings`
- `cargo test --offline`: 380 passed, no failures
- `cargo test --offline --no-default-features`: 258 passed, no failures
- `git diff --check`

The final focused `rotation_preference` run also passed all seven tests, including
explicit opt-in on immutable completion originals. No changes to CI, security
configuration or agent instructions. Release performance comparisons were not run.

## Review correction

The profile jump-optimizer adapter now forwards `self.rotation_preference`
instead of replacing it with the default. A direct adapter regression exercises
both settings: it failed on `AllowMixedDirections` before the fix and passed after.

Focused validation passed: four jump-optimizer unit tests, seven public jump-optimizer
tests, twelve profile tests and seven rotation-preference tests (30 total), plus
Clippy with warnings denied, formatting and diff checks. Both Standards and Spec
reviewers cleared the correction. The full suites above were run for the original
implementation; they were not repeated for this one-line adapter correction.
