# Six-profile theoretical planning readiness (#117, #125)

The complete native public planning workflow passed the integrated acceptance
matrix for Adult Bronze, Silver, Gold and Masters and Senior Women and Men.
All 72 repeated primary searches completed with the expected outcome. All 144
deterministic lifecycle cancellation runs retained the required proof boundary.
No measured request hit the declared external 120-second watchdog or returned
an assessment, solver or process error.

This report records the pre-#132 baseline. Silver values below are historical;
current source-complete Silver maxima are 14.80 (empty/excluded) and 14.70
(required compound and CSSp2). See [current acceptance](milestone-132.md).

## Public workflow and correctness evidence

`selectable_elements`, `selectable_jump_elements`, `assess_manual_bv` and
`search_maximum_bv` share the selected profile and table. Search builds its
complete repertoire, preserves exact freely reordered required occurrences and
their stable IDs, applies semantic exclusions to every compound component, and
returns up to five distinct proved maximum-valued whole-element multisets.
Five results carry LimitReached, not a claim that only five exist. Exhausted
requires a proof that no more compositions remain.

The independent source evidence and public regressions remain in:

| Evidence | Public behavior checked |
| --- | --- |
| `tests/planned_jump_domain.rs`, `tests/fixtures/planned-jump-grammar.tsv` | Independently expanded complete six-profile grammar: 30/849/2652/6570/51518/51518 jump atoms; mixed sequences, Euler, Senior permitted solos and forbidden quint compounds. |
| `tests/element_selection.rs`, `tests/theoretical_manual_planning.rs`, source SOV fixtures | Profile levels, intended V/full values, named spin forms, literal manual values, Senior 80% credit and source coverage. |
| `tests/planning_search.rs` | Adult literal maxima, exact Silver 1F+1Lo/CSSp2, reversed requirements, duplicate occurrences, solo/compound distinction, exact-rotation and named-spin exclusions, impossible roles and source-derived small exhaustive oracle. |
| `tests/planning_search_senior.rs` | Both complete Senior maxima, exact contextual objective, required V/Euler choices and raw-versus-credited winner. |
| `tests/planning_alternatives.rs` | Independent atomic partition oracle; one, fewer than five, exactly five and more than five maxima; required-ID/reordering identity; partial alternative retention. |
| `tests/planning_readiness.rs` | Six-profile duplicate exact requirements plus global ToeLoop exclusion, per-alternative provenance/assessment, indexed conflicts and four-Flip impossibility. |

The count-model and contextual-credit equivalence argument remains in the
[search contract](specs/planning-search.md). Every reconstructed candidate is
checked by shared manual assessment; this consistency check supplements the
independent fixtures and worked bounds.

IncumbentFound now follows successful shared reconstruction and manual
assessment. A failing assessment propagates as an error, and cancellation during
that reconstruction emits no incumbent event. The TDD tracer first failed on
the former raw notification (one event instead of zero), then passed with the
shared reconstruction in the optimiser callback. The phase still publishes no
program or value. Before proof, cancellation returns no program. After proof,
it retains the validated maximum and confirmed alternatives with Cancelled
alternative status. No default time or node budget was added.

## Repeated native Release experiment

Measured source revision: `5279ff11bc4beba261875aa61a1a54b64d38cca5`.
The subsequent changes add the verifier, report and retained artifacts only.
The [runner and metric definitions](../benchmarks/planning-readiness/README.md)
describe reproduction and phase boundaries. The
[complete summary](../benchmarks/planning-readiness/observations-2026-11-08/summary.md)
contains all 24 primary scenario rows and every measured timing column.
The [CSV](../benchmarks/planning-readiness/observations-2026-11-08/observations.csv),
24 request manifests, exact Cargo.lock, environment and raw stdout/stderr archive
are retained beside it. The verifier checked all 216 rows, identical request
hashes, literal expected maxima, proof and alternative statuses, positive OS
peak memory and cancellation outcomes.

Environment: Windows 11 Pro 10.0.26200 64-bit; Intel Core i9-13900K, 24 cores /
32 logical processors; 68,414,652,416 bytes RAM; Rust 1.99.1
(`58cf8f9c3`, 2026-10-13), Cargo 1.99.1, LLVM 23.1.1,
`x86_64-pc-windows-msvc`; Pumpkin 0.5.0. The checked-out toolchain override
selected these versions for both compilation and the recorded inventory.
The binary SHA-256 is
`B5A7E6906EC6EF49E4E3AE5548A1BEB83AAFEC58E5AE7E56836E7FF1D2691952`.

Each request ran three times in a fresh process, serially, with no compiler or
test jobs running and no excluded warmup. All feasible primary requests proved
their maximum and returned five validated alternatives with LimitReached.
All impossible requests returned supported preflight Infeasible; absent later
phase metrics remain blank. No repertoire was reduced and no request was
substituted after observing a result.

The empty-request results are:

| Profile | Maximum BV | Median total ms | Total range ms | Maximum peak MiB |
| --- | ---: | ---: | ---: | ---: |
| Bronze | 11.00 | 9.654 | 8.993–9.955 | 5.98 |
| Silver | 14.30 | 19.337 | 19.171–19.778 | 7.91 |
| Gold | 24.20 | 39.262 | 38.387–40.802 | 12.72 |
| Masters | 33.40 | 110.720 | 103.285–111.369 | 23.38 |
| Women | 133.10 | 3046.566 | 2774.865–3282.770 | 150.26 |
| Men | 133.10 | 2897.961 | 2816.828–3015.176 | 150.54 |

Across all primary requests the slowest return was 3282.770 ms and the largest
process peak was 150.69 MiB. These are observed values on one host, not latency
or memory guarantees. Peak memory is Windows lifetime PeakWorkingSet64 read
while the child waits after its final metric line; the ACK handshake is outside
API total time. Phase instrumentation is included in the measured API time.

Two setup launches ended before any measured API call: the expected-revision
guard rejected a mistyped revision, then the sandbox denied read-only Windows
CIM inventory. The corrected runner executed with approved native inventory
access. These setup attempts are recorded in `setup-attempts.txt`; neither was
reported as infeasibility or omitted from a completed measurement matrix.

## Independent expected values for the measured requests

Values below are integer hundredths from the source fixtures and category
limits, rather than values copied from production assessment. Empty Adult
jump bounds are 270, 490, 1080 and 1780; best permitted non-jump totals are
830, 940, 1340 and 1560. The public tests exhibit legal attaining compositions.

Bronze required 1F and global ToeLoop exclusion retain 1100: the best five jump
components use two 1Lz and three 1F/1Lo. Silver required 1F+1Lo fits the best
seven-component inventory, but CSSp2 replaces CLSp2 at a loss of ten hundredths,
giving 1420. Silver ToeLoop exclusion retains 1430.

Gold required 1A retains 2420. Excluding ToeLoop removes both 2T components from
the 1080 jump bound; replacing them with 1Lz gives 940, hence 2280 overall.
Masters required 1A leaves the best seven other components as two each of
2A/2Lz/2F and one 2Lo: 1720 including 1A, hence 3280 overall. Masters ToeLoop
exclusion retains 3340.

For either Senior profile the six jump entries permit nine listed components,
with quints restricted to solos. Four distinct quints contribute 5600; two 4A
contribute 2500; 4Lz/4F/4Lo contribute 1150/1100/1050. This attains the 11400
jump bound without solo-repetition loss, plus 1910 non-jump BV, giving 13310.
Required solo 1A occupies a quint slot: 11400 - 1400 + 110 + 1910 = 12020.
ToeLoop exclusion leaves four distinct permitted quint types and the same
attaining compound inventory, so 13310 remains achievable.

Four required 1F solos exceed Adult identical-jump repetition limits (two) and
Senior jump-kind totals (three). They establish direct infeasibility in every
profile without an interrupted search or an assumed optimum.

## Cancellation and checks

Every profile ran three repetitions at Domain, Model, Solve, IncumbentFound,
Reconstruction, MaximumProved, Alternatives and later alternative reconstruction.
All pre-proof cancellations returned Cancelled with no program; all post-proof
cancellations retained the known maximum and already validated alternatives.
The worst trigger-to-return observation was 94.142 ms in Men during later
reconstruction. This measures cooperative response after a cancellation poll;
it does not measure the wait until the next poll or promise instant cancellation.
The complete per-phase table is in the retained summary.

Required checks passed: build, formatting, clippy over all targets with warnings
denied, default suite (372 tests), minimal-feature suite (254 tests), and the
focused Release readiness tests. The default suite retains legacy immutable
ordered completion, insertion/provenance, supplied recognition and detailed
evidence regressions. The PowerShell parser, metric schema and complete
observation verifier also passed.

No unresolved readiness failure remains in this measured matrix. It covers
these six native profiles and the declared requests. Background host load and
thermal state were uncontrolled; no UI, service, browser or WASM readiness was
tested or claimed. Performance recognition and feature evidence retain their
documented scope and partial coverage.
