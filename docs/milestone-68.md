# Product workflow acceptance (#68)

This audit covers the native Rust 2026/27 Adult Single Free Skating model.
Dependencies #64–#67 merged before this audit (baseline `27dd01e`). Passing these
fixtures establishes the stated conditional workflows, not complete ISU legality.

## Acceptance matrix

| Product function | Shared entry points and executable evidence | Remaining limits |
| --- | --- | --- |
| Manual BV and optional GOE | `assess_manual_program`; `tests/assessment.rs::bronze_and_silver_source_checked_bv_and_missing_goe`, `absent_zero_and_mixed_goe_are_distinct`, `supplied_calls_falls_and_unsupported_outcomes_stay_separate`, `unknown_calls_and_unsupported_combinations_prevent_exact_scenarios` | Intended clean recognition and separately supplied scenarios. None is never fabricated as GOE zero. #56/#58 remain unsupported. |
| Manual legality | `assess_program_legality`; `tests/legality.rs::gold_prohibition_survives_missing_content_and_draft_omissions_are_indexed`, `all_four_profiles_keep_distinct_obligations`, `sequence_without_direct_step_assumption_and_repeated_components_stay_visible`, feature/foot/prerequisite cases in that suite | Draft absences differ from final violations. Source-bound evaluated, conditional, missing and unsupported rules coexist. Coverage is partial even with no violations. |
| Generate complete bounded optimum | `generate_program_candidates` → `optimize_program` → manual assessment and legality; `tests/workflow_acceptance.rs::generated_mixed_optimum_matches_every_independently_enumerated_order_and_rechecks` | No handwritten variant list. Independent literal SOV coefficients and exhaustive words establish BV 520 and all 12 ordered ties for the declared three-element Bronze domain. This does not enumerate all physical programs or maximize a full-length event program. |
| Complete an ordered partial program | generator → `complete_program` → manual assessment and legality; `generated_completion_preserves_two_anchors_and_every_oracle_tie` | Two immutable anchors, all insertion gaps, all visible ordered ties and every distinct anchor placement are compared with independent enumeration. Global feature/repetition, exclusions, missing roles, prohibited locks and impossible requests: `tests/program_completion.rs`. |

The new oracle never calls a production validator or scorer to decide expected
words or objective values. It enumerates base-four words using separately
transcribed coefficients 40, 40, 130, 350 from Communication 2786 (June 2),
1T/1S p.2, SSpB p.6 value row 16, ChSq1 p.14 value row 11.
Every returned program is then independently passed through the shared public
assessment and legality APIs; this second pass checks integration, not the oracle.
Metamorphic checks exclude a harmless extra Lutz and narrow eligible jumps.

The anchor oracle exposed an existing deduplication defect: identical originals
and additions shared equivalence keys. Completion now keeps original IDs distinct
while equivalent addition definitions still share a key. The 24 provenance-aware
optimal completions are checked independently, including the cases previously lost.

## Four independent completion questions

| Question | Evidence / interpretation |
| --- | --- |
| Is the mathematical objective proven? | `OptimizerStatus::OptimalProven` proves the maximum over supplied finite variants and implemented constraints only. `FeasibleUnproven` is exercised by `optimizer::programs::tests::stopped_search_returns_unproven_incumbent`. `Incomplete` has no proven objective. `InfeasibleProven` excludes solutions only within that domain. |
| Is candidate generation complete? | `CandidateDomainCompleteness` concerns the declared supported generation selections only. Candidate/expansion caps and omissions are reported separately. `tests/candidate_generation.rs` and completion's generated-domain audit test cover this distinction. A complete generation audit can still include explicit unsupported omissions. |
| Are all optimal alternatives returned? | `enumeration_complete` is independent of objective proof. `false` means further optima may exist; returned length is not a total tie count. `tests/workflow_acceptance.rs` compares every ordered optimum and tests a one-result cap. Existing completion tests cover identical assignments and changed original recognition assumptions. |
| Are all applicable regulations evaluated? | No. Every result retains partial regulatory coverage, source-bound assumptions and unmodeled rules; see `tests/legality.rs`. Numerical catalog support does not establish category eligibility or recognized execution. |

There is no pagination cursor or streaming API. A capped finite batch is supported.
Consumers must expose that further alternatives may exist. Repeating a request
with a larger cap restarts the search; it is not a continuation and does not promise
stable batch order. Do not sum batch lengths to estimate total ties.

## Errors and arithmetic

`finite_bounds_and_identity_errors_never_produce_a_plausible_optimum` rejects
unknown profiles, out-of-range bounds, unsupported Pair scope and absent recognition.
Unknown table editions, unsupported levels/calls and evidence: `tests/profiles.rs`,
`tests/catalog.rs`, `tests/scoring.rs`, `tests/assessment.rs`, `tests/legality.rs`.
The source catalog's independent fixture audit checks exact positive and negative
row identities; `tests/scoring.rs` covers signed integer GOE rounding.

Program BV is an unfactored nonnegative integer sum. Each source value fits `u16`;
ten slots sum to at most 655350, below `u32` and solver `i32` bounds. The solver's
objective upper bound is at most ten times the largest candidate value. Existing
caps make integer overflow impossible on this path. This argument does not cover
future larger bounds or GOE/second-half objectives.

## Measured bounds and default-cap decision

Run `cargo run --example workflow_bounds` and
`cargo run --release --example workflow_bounds`. The example uses generation,
explicit competing IncreaseOfSpeed spin evidence, combinations, basic change-foot
combination/flying spins, movement obligations and six ordered slots. No library
API or CI changes are needed. Native measurements below exclude compilation and
include validation, model construction and search. The harness emits individual
CSV rows; raw observations are in `workflow-bounds-debug.csv` and
`workflow-bounds-release.csv`.

Context: 2026-11-03; Windows x86_64 MSVC; rustc 1.99.1
(`58cf8f9c3`, LLVM 23.1.1), Pumpkin 0.5.0; Intel64 Family 6 Model 183
Stepping 1, 32 logical processors reported by environment. CPU marketing name,
physical memory and peak RSS were unavailable in the sandbox. No memory or WASM
claim is made. Runtime baseline is `27dd01e`; harness and fixture source are the
issue-68 PR revision containing this report. One observation per configuration,
not a statistical benchmark. Other host work and timing variance remain uncontrolled.

Each run uses 200 validator proposals, a 250 ms cooperative deadline, and result
caps 1, 10 and 100. All twelve runs in each build return `Incomplete`,
`enumeration_complete=false`, no candidates and `Interrupted`.

| Profile | Generated variants | Debug generation ms | Debug search ms (range) | Debug checked proposals | Release generation ms | Release search ms (range) | Release checked proposals |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Bronze | 19 | 5.404 | 250.587–251.106 | 30 | 2.325 | 251.229–251.271 | 66–70 |
| Silver | 24 | 5.663 | 250.647–251.013 | 13 | 2.138 | 251.416–251.456 | 43 |
| Gold | 49 | 6.340 | 250.342–251.866 | 5–7 | 2.794 | 251.062–251.253 | 33 |
| Masters | 64 | 7.027 | 250.567–250.948 | 4 | 2.781 | 250.692–250.798 | 55–56 |

Result-cap sensitivity is unobservable in these broader runs: no incumbent is
found before the deadline. The small independent fixture proves the cap's effect
on tie reporting (1 returned versus 12 with exhaustive enumeration), without a
latency guarantee. Broad domains expose the cost of post-validation of optimistic
solver assignments; increasing a result cap does not address this bottleneck.

Actual accepted request ceilings remain 1000 definitions, ten slots and 100000
checked proposals. These are validation ceilings, not safe latency defaults.
`max_proposals` counts proposals checked by the shared validator, not internal
Pumpkin nodes, and does not preempt a solver call. The deadline participates in
Pumpkin's termination callback, but preflight/model construction and callback
frequency mean it is not a hard wall-clock guarantee. The observed overshoot is
visible above. No unrestricted Cartesian-product expansion is introduced.

Decision: retain existing ceilings and solver behavior. For an interactive trial,
start with explicit 250 ms / 200 proposal / 10 output limits, accept `Incomplete`
as a normal outcome, and require smaller personal domains to obtain useful results.
These measurements do **not** establish adequate interactive performance for
six-slot broad domains. No universal safe success-producing default exists in
this evidence. Native batch users must choose explicit time and output budgets.
Further pruning needs its own measured implementation and independent rule tests;
this acceptance audit does not silently duplicate skating rules in solver clauses.

## Senior follow-up (#81)

The [Grand Prix benchmark report](senior-grand-prix-benchmarks.md) adds pinned,
offline source-derived Women/Men domains from France and Canada at three scales,
both full and immutable-completion paths, independent optimum/tie oracles, and
250 ms / 1 s / 5 s / 30 s Debug/Release observations. The historical Adult data
above stays unchanged. Its six-slot 19–64-variant results do not establish a
speedup comparison against different Senior inputs. Eleven-slot Senior capacity
is an engineering limit; bounded generation, objective proof, ordered tie
exhaustion and regulatory coverage remain separate claims.

