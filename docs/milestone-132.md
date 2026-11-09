# Backend planning acceptance #132 / #138

The acceptance scope is complete theoretical category construction, credited BV,
exact required occurrences and personal repertoire restrictions through the public
search and manual-assessment APIs. Duration, actual execution, GOE/PCS optimization
and UI are outside this scope. This report does not substitute partial validation
for the binary `planning_compliance()` verdict.

## Reproducible acceptance matrix

`tests/backend_acceptance.rs` runs unrestricted, required-only, restricted-only
and combined requests for all six built-in categories. Every returned alternative
has a distinct sorted whole-element identity, exact required provenance and
independent literal credited BV, passes fresh manual assessment and has all eight
source-linked inventory families evaluated. ToeLoop exclusion checks every compound
component. The required Silver compound stays whole, alongside CSSp2.

| Category | Unrestricted | Required-only | Restricted-only | Combined |
| --- | ---: | ---: | ---: | ---: |
| Bronze | 11.00 | 11.00 | 11.00 | 11.00 |
| Silver | 14.80 | 14.70 | 14.80 | 14.70 |
| Gold | 24.20 | 24.20 | 22.80 | 22.80 |
| Masters | 33.40 | 32.80 | 33.40 | 32.80 |
| Senior Women | 133.10 | 120.20 | 133.10 | 120.20 |
| Senior Men | 133.10 | 120.20 | 133.10 | 120.20 |

Required cases: Bronze solo 1F; Silver exact 1F+1Lo and CSSp2; remaining profiles
solo 1A. Restricted cases exclude ToeLoop globally. The independent source bounds
are in [planning compliance](specs/planning-compliance.md),
[the six-profile report](milestone-117.md) and the existing bounded partition
oracles. Silver changes because the July16 announcement p10 permits a second
spin of any nature: ChSp1(350) replaces CCoSp2(300). The maxima are
jumps490 + CLSp2(290), or required CSSp2(280), +ChSp350 +ChSq350.

`tests/registered_planning.rs` separately registers a two-jump synthetic category
with explicit synthetic sources: unrestricted 710, required 1T 690, restricted 670,
combined 670 hundredths. Independent manual `[1T,1T,CLSp1,ChSq1]` is compliant at 670.
The twelve-slot regression proves extension beyond eleven without changing the
separate ordered-completion limit. An unsupported registered mechanism yields an
unsupported profile error at both public seams, never infeasibility or compliance.

## Rule and workflow evidence

| Requirement | Public-seam evidence |
| --- | --- |
| Binary manual compliance and all source families | `manual_bv::binary_construction_covers_source_rule_families_for_all_six_profiles`, `backend_acceptance::independent_manual_mutations_reject_source_construction_rules`; independent legal bases and role-removal, compound-length/budget, repetition, spin abbreviation, step and choreography mutations with source-linked violations |
| Input errors, unsupported input, independent personal restrictions | `manual_bv::complete_planning_verdict_checks_input_before_category_rules`, `understood_invalid_shapes_are_noncompliant_and_missing_intentions_are_errors`, `theoretical_manual_planning`, `registered_planning` |
| Exact and semantic exclusions across all families | `planning_search::exact_exclusions_cover_every_family_without_excluding_other_levels_or_forms`, `exact_whole_exclusions_preserve_components_and_localize_duplicate_requirements`, `semantic_exclusions_cover_compounds_rotations_and_named_spin_forms` |
| Every personal maximum and absent restrictions | `personal_limits::every_spin_maximum_retains_all_lower_levels_and_choreography`, `every_jump_kind_uses_listed_classes_and_category_ceiling`, `step_limits::selection_retains_lower_steps_and_intersects_category_ceiling`, `search_caps_steps_with_other_limits_exclusions_and_exact_requirements` |
| Required multiplicity, conflicts and infeasibility | `planning_readiness`, `planning_search`, `planning_search_senior`, `personal_limits`, `step_limits`; exact whole-element provenance, duplicate occurrence IDs and localized conflicts |
| Independent optimality and alternative identity | `planning_alternatives`, `planning_bound_pruning_oracle`, `planning_search::complete_public_repertoire_matches_an_independent_small_exhaustive_oracle`; bounded independent enumerators, no second production engine |
| Fewer/exactly/more than five, no reordered duplicates | `planning_alternatives`: two exhausted/one duplicate-required optimum, exactly five LimitReached, more-than-five independent bounded oracle; four positive Bronze required-F partitions; ignoredW yields unbounded neutral multiplicity |
| Truthful proof, alternatives and cancellation | `planning_readiness::an_incumbent_notification_requires_completed_manual_assessment`, `planning_search::cancellation_before_proof_exposes_no_program_and_after_proof_retains_it`, both alternative-reconstruction cancellation tests; native matrix cancels at all eight lifecycle boundaries |
| Waltz identity, source counting and neutral alternatives | `waltz_planning`: ignored Bronze, counted Silver/Gold, explicit Masters/Senior classifications; no invented neutral ceiling |
| Ordered completion and detailed evidence remain independent | Entire `program_completion`, `execution_scoring`, `spin_validation`, `program_validation`, `planned_v_spins` and scoring suites in the full default/minimal checks |

All eight admitted construction families, their exact source editions, category
differences and exact-code recognition intentions are documented in
[the audited positive inventory](specs/planning-compliance.md). Manual mutation
tests include removal of each Gold/Masters/Senior required spin role and Axel
where required. Separate detailed evidence APIs keep their original evidence
requirements; planned code recognition does not supply performed evidence.

## Verification and measurements

Technical acceptance passed. All required checks and 216 current native Release
processes passed, with no watchdog failure, solver/assessment mismatch or unrun
required check. The final full default suite passed 443 tests; minimal features passed 274.
The separate solver comparison passed both independent fixture/reference tests
and both Pumpkin/Z3 runs (proven120 and infeasible under exclusion).

Final verified and measured revision:
`d298c274ace7d97beaff599bda7675a6b34afd0c`, including review fix `3eccb82`.
All five checks, the separate solver run/tests, Release build and 216-process
measurement were rerun after that fix. The acceptance worktree fast-forwarded
to this integration tip before verification; the final integration merge check
was already up to date. Later commits contain this report and evidence only.
The initial acceptance revision `81f63b3` and measurements at `79229ad` remain
historical evidence, not the current snapshot.

| Command | Result |
| --- | --- |
| `cargo build --offline` | exit 0 |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --offline --all-targets -- -D warnings` | exit 0 |
| `cargo test --offline` | exit 0; 443 passed |
| `cargo test --offline --no-default-features` | exit 0; 274 passed |
| `cargo run --offline --manifest-path solver-comparison/Cargo.toml --locked` | exit 0; `CARGO_BUILD_JOBS=2`, fresh native target, escalated |
| `cargo test --offline --manifest-path solver-comparison/Cargo.toml --locked` | exit 0; 2 passed, same environment |
| `cargo build --offline --release --example planning_readiness` | exit 0 |
| `pwsh -NoProfile -File benchmarks/planning-readiness/run.ps1 -ExpectedRevision <measured-revision> -OutputDirectory ../issue-138-final-native-release -Runs 3 -WatchdogSeconds 120` | exit 0; 216 passed |
| `python benchmarks/planning-readiness/summarize.py ../issue-132-baseline baseline-caa4127` | exit 0; 216 verified |
| `python benchmarks/planning-readiness/summarize.py ../issue-138-final-native-release` | exit 0; 216 verified |
| `python benchmarks/planning-readiness/compare.py ../issue-132-baseline ../issue-138-final-native-release` | exit 0; 24 byte-identical manifests |

Evidence is retained in
[`observations-issue-132`](../benchmarks/planning-readiness/observations-issue-132/):
baseline/current CSVs, environments, 24 request manifests each, exact root locks,
432 raw stdout/stderr files per edition in ZIPs, full summaries, the proof versus
alternatives comparison and compressed verification logs. `final-verification-logs.zip`
contains the final checks, exact separate solver-comparison lock, review-fix logs
and historical sibling native-build failure. `verification-logs.zip` retains the
initial checks/setup failures; `historical-current-79229ad.zip` retains the entire
pre-review current snapshot. The baseline snapshot is unchanged. To reverify archived observations,
run `summarize.py <baseline-dir> baseline-caa4127`,
`summarize.py <current-dir>` and `compare.py <baseline-dir> <current-dir>`.
The original baseline at `caa4127cdd1215d618e1a75b1cb96cd46d8e877a`
retains its old values and outcomes. Both root locks have SHA256
`4687dff3c4302e35738ed1b09fdc9dd9df47c6b076a3b42ab573099b5570f9e8`.

All measurements used the same Windows 11 Pro 26200 host, i9-13900K (24 cores,
32 logical), 68,414,652,416 bytes RAM, Rust 1.99.1 and native MSVC Release.
The coordinator held other compiler/test/benchmark jobs. Three fresh serial
processes per request, no excluded warmup, no API budget; an external 120 s watchdog
recorded no failures. Peak working set, binary hashes and exact toolchain versions
are in each environment/CSV. No processes were left running.

Performance regressions are material. Silver empty proof median rose
16.019→1195.972 ms (74.66×), excluded 15.295→1059.006 ms (69.24×), and required
16.071→53.882 ms (3.35×), alongside the corrected larger 14.80/14.70 maxima.
Silver alternative collection rose 2.621→8.287 ms (empty), 2.192→6.814 ms (excluded)
and 2.517→5.227 ms (required). This experiment does not isolate causes between
the policy/model/assessment changes. Inspection of `value_upper_bound` shows an
optional ChSp is added independently while regular-spin slots are reserved only
for required ChSp, leaving a loose Silver upper bound. That is a potential
contributor, not a measured causal attribution. Other proof ratios ranged 0.99–1.40×;
Masters empty proof 41.473→54.276 ms and alternatives 13.372→23.078 ms.
The deterministic Masters collection regression budget also changed from 14,000
to 20,000 polls in #136; it remains a caller-controlled cancellation test,
not an optimization SLA. Bronze empty alternatives 1.134→2.241 ms now include
neutral W augmentation; the required case changes four Exhausted into five LimitReached.
The full comparison reports every primary case separately. Cancellation response
over all 144 requests was at most 8.459 ms; this is an observation, not a guarantee.

Two setup failures preceded measurement: an incorrect ExpectedRevision argument
was rejected before setup, and sandbox CIM hardware access failed before any
workload. A fresh escalated run collected the complete matrix. Original summary
verification also failed on baseline Bronze required 1F because its blanket
five-alternative assumption was obsolete. The independent four-multiset regression
now proves that baseline result; versioned expectations preserve all other checks.
All failure logs are retained. The earlier sibling native archive access-denied
failure was avoided by the fresh escalated two-job comparison build.

Review fixed loss of already confirmed alternatives when ignored Waltz augmentation
was cancelled and restored required-occurrence IDs. The new public-seam regression
retains both collected maxima, exact required Waltzes/provenance, fresh compliant
manual assessments and `Cancelled` alternative status. An obsolete fixture that
expected five alternatives despite cancellation now expects the already confirmed
one. Its failed focused run and corrected pass are both retained in the review
logs. The shared internal mechanism predicate was also consolidated. Final Clippy
exited 0 under `-D warnings`; its log includes environmental incremental-cache
hard-link failures with successful file-copy fallback, rather than code lint findings.

The backend satisfies the four planning workflows within the documented
construction scope. Native performance regressions remain visible; this acceptance
adds no SLA and establishes no browser/WASM or UI latency claim.

## Compatibility approval

The user explicitly approved both compatibility decisions on 2026-11-09 in the
requesting chat: the ExactElement, StepLevel, Waltz and AdultWaltzPlanV1 enum
additions, and the narrowly scoped profile-version exception for the listed #132
corrections under unchanged authoritative editions. External exhaustive Rust
matches need updating; prior calculated results need recomputation.

The [accepted compatibility decisions](specs/issue-132-compatibility-proposal.md)
and [ADR 0002](adr/0002-rule-profiles-proposal.md#approved-correction-exception-132)
record the exact scope. Both repository-required approval gates are closed.
