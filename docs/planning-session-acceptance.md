# Complete planning-session acceptance (#140, #143)

The approved [public contract](specs/planning-search-session-proposal.md) adds
complete session access alongside legacy five-result delivery. A maximum becomes
observable only after proof and validated representative assessment. Each later
batch contains new compositions; a full batch means collection can continue.
Exhaustion requires an unsatisfiable fixed-maximum solve. Pause retains confirmed
results and can resume only in the same open session. Cancellation is terminal.
No persistence, UI, HTTP, duration assessment or execution recognition is added.

## Independent acceptance evidence

| Public-boundary evidence | Requirement and independent expectation |
| --- | --- |
| `backend_acceptance::six_category_four_workflow_matrix_has_independent_credited_bounds` | Six profiles × unrestricted/required/restricted/combined; at least one delivery and continuation; source-derived BV maxima, exact required IDs and compound ToeLoop exclusion; fresh manual BV and complete compliant rule inventory. |
| `registered_planning::registered_category_supports_all_four_planning_workflows` | Registered two-jump policy, all four workflows to exhaustion; independently worked 710/690/670/670 bounds and fresh manual assessment. |
| `planning_alternatives::more_than_five_maxima_match_an_independent_bounded_atomic_oracle` | Complete independently enumerated optimum multiset set at1070, distinct compound orders, no duplicates, legacy subset capped5. |
| `planning_alternatives::session_filled_batch_requires_a_separate_exhaustion_proof` and lifecycle tests | Filled batch, later exhaustion, required occurrence preservation, partial/empty pause and reconstruction continuation, retained results, cancellation and terminal errors. |
| `planning_alternatives::session_resumes_after_pause_at_every_collection_checkpoint` | Sweeps every cooperative checkpoint prefix through exhaustion, including solver termination polls; resumes the same session and preserves exactly the two independently known Bronze ToeLoop compositions at 910, without duplicates or poll-count expectations. Fresh manual BV checks every delivery. |
| `planning_alternatives::registered_ignored_mechanism_exhausts_required_waltz_compositions` | Registered mechanism, two retained caller-identified ignored Waltzes; exactly two compositions at910. |
| `planning_alternatives::counted_zero_value_waltzes_remain_distinct_optional_elements` | Counted Silver zero-BV Waltz multiplicities0/1/2; exactly three compositions at980 through both APIs. |
| `planning_alternatives::session_required_compound_is_indivisible_and_exact_exclusion_keeps_internal_order_distinct` | Exact compound and whole-code exclusion, retained multiplicities and provenance. |
| `step_limits::search_caps_steps_with_other_limits_exclusions_and_exact_requirements` | Combined exact requirement, semantic exclusion, jump/spin/step ceilings; fresh manual checks, spin-form infeasibility and indexed conflict stay distinct. |
| `rotation_preference::automatic::silver_proves_1480_in_both_modes_and_preserves_required_conflicts` | Both rotation modes; exact required compound conflict remains indexed and opt-in succeeds through session proof. |
| `registered_planning::unsupported_registered_mechanism_never_becomes_infeasibility_or_compliance` | Unsupported and unknown profiles produce Failed with profile errors, no maximum or deliveries. |
| `backend_acceptance::independent_manual_mutations_reject_source_construction_rules` | Six independently entered complete manual plans and source-bound single-rule mutations; no solver-derived manual inputs. |
| Existing manual, personal-limit and step-limit tests | Malformed/draft/unsupported distinctions and unchanged numerical BV/competition findings when personal conflicts are added. |

The built-in matrix uses literals from Adult July16 pp8–11, Rule612 pp70–72
and Communication2786 June2: Bronze1100; Silver1480 or required1470;
Gold2420 or ToeLoop-restricted2280; Masters3340 or required3280;
Senior Women/Men13310 or required12020. These are independent worked bounds,
not captured solver outputs. Unrestricted advanced requests collect a practical
prefix; no unrestricted exhaustive-runtime claim follows from this test.

`examples/planning_sessions.rs` checks the independent finite oracle to exhaustion
in uninterrupted and paused/resumed modes. The runner compares complete identity
sets across both modes and every process. Fresh manual reassessment checks exact
BV, complete construction, personal restrictions and stable required provenance
outside timed API boundaries. This demonstrates full multibatch equivalence
without depending on collection order or private solver structure.

## Native session measurements

The checkpoint sweep first reproduced a Pumpkin 0.5.0 panic when resuming a
root-level timeout. Collection now latches requests at every termination poll
and defers stopping until a branching decision supplies a safe non-root
checkpoint. Backtracking clears that witness; the retained solver, brancher and
resolver survive the call. This adds no response-time bound.

The [measurement protocol](../benchmarks/planning-sessions/README.md) defines
reproduction commands and observable boundaries. On 2026-11-09, native-host-A
(Windows 10.0.26200, Intel64 Family 6 Model 183 Stepping 1, 32 logical processors),
rustc 1.99.1 / Cargo 1.99.1, x86_64-pc-windows-msvc, Pumpkin 0.5.0,
ran three fresh serial Release processes per session mode after all compiler/test
jobs finished. Source revision was `4d79c128d04a2214154bf0afb2bdd2f1640da95d`,
including the reviewed solver-pause correction.
The [environment record](../benchmarks/planning-sessions/observations-issue-140-session/environment.txt)
contains binary and lock hashes. No warmup was excluded.

At the requesting user's direction, the native host is published under the alias
`native-host-A`. Unpublished commits and their revision references were rewritten
only to anonymize this metadata. Compiled Rust sources, tests, examples, binary
hashes, request manifests and measured values are unchanged.

Every process proved 1070, collected exactly 16 distinct compositions in five
batches and proved exhaustion. Complete identity hashes match across all six
processes and both modes. Same-mode request manifests match byte-for-byte.
Pausing after five deliveries retained confirmed results; resumption collected
the remaining set without duplicates.

| Median of three processes | Uninterrupted | Paused/resumed |
| --- | ---: | ---: |
| Session construction | 2 microseconds | 2 microseconds |
| Maximum proof including initial assessment | 8.929ms | 8.474ms |
| First five deliveries | 1.580ms | 1.565ms |
| Active collection through exhaustion (16 deliveries) | 5.148ms | 5.343ms |
| First resumed batch (three deliveries) | - | 0.897ms |
| Process lifetime peak resident memory | 6,725,632 bytes | 6,742,016 bytes |

The pause call measured below 1 microsecond at integer-microsecond resolution
in each process; this is not a zero-cost or response-time guarantee. Active
collection excludes the pause call and independent manual/oracle checks. No
consumer idle delay was inserted. Every batch duration, size and state, complete
request, identity set, stderr, process exit and raw observation are retained in
[session observations](../benchmarks/planning-sessions/observations-issue-140-session/observations.csv).
Complete finite-set exhaustion performs more work than five-result delivery and
is reported separately.

## Legacy regression comparison

The same host and toolchain ran 144 fresh serial Release processes against parent
baseline `c295da41527bb2fe13353f4138865e56f4e9b143` and integrated implementation
`4d79c128d04a2214154bf0afb2bdd2f1640da95d`: six profiles, four scenarios
(empty, required, excluded, impossible), three runs and two revisions. All 144
passed. The runner compared each full request manifest byte-for-byte across both
revisions and all repetitions. The summary verifier checks 144 unique rows,
the complete matrix, manifest hashes, independent maxima, statuses and exits.
Both builds used the same generated lockfile. Binary hashes, lock copies and
provenance, machine, full toolchain, requests and stdout/stderr are retained with
[raw legacy observations](../benchmarks/planning-sessions/observations-issue-140-legacy/observations.csv).

The [median summary](../benchmarks/planning-sessions/observations-issue-140-legacy/summary.csv)
uses three processes per request and revision. The largest observed proof
regression was Masters excluded: 52.372 to 53.827ms (+2.8%, 1.455ms).
The largest collection regression was also Masters excluded: 10.613 to 11.519ms
(+8.5%, 0.906ms); its total rose from 62.786 to 65.059ms (+3.6%). These
observations do not identify a causal change or establish a latency threshold.
No warmup was excluded. Background host load and thermal conditions were
uncontrolled. All measurements were refreshed after the solver-pause production
fix. The earlier pre-fix batch at `913ba5b` observed maxima of +8.3% proof time
(Senior Men required) and +19.3% collection time (Senior Men excluded). Its raw
data and report remain in commit `fa91577`; it was not discarded as a warmup.
The batches use different implementation revisions and do not establish a causal
performance change.

Bronze required changed from five nominal alternatives in the baseline to four
distinct exhausted compositions after ignored-movement normalization. Collection
timings there cover different delivered amounts. The session finite oracle also
performs different work from this legacy matrix. Native results establish no
browser, WASM, UI or latency guarantee. Build/run and median-recomputation commands
are in the [measurement protocol](../benchmarks/planning-sessions/README.md).

## Prescribed checks

These commands passed after the reviewed solver-pause correction `428395e`:

| Check | Result |
| --- | --- |
| `cargo build --offline` | Pass |
| `cargo fmt --check` | Pass |
| `cargo clippy --offline --all-targets -- -D warnings` | Pass |
| `cargo test --offline` | Complete default-feature suite passed, including the new checkpoint sweep; 0 failed |
| `cargo test --offline --no-default-features` | 274 passed, 0 failed |
| Focused public alternative/session tests | 16 passed, 0 failed; earlier cross-workflow focused run: 32 passed |
| `cargo build --offline --release --example planning_sessions` | Pass |
| Six fresh-process session examples | Pass; all 16 identities and independent manual checks in every process |
| `git diff --check` | Pass |

The parent integration additionally checked solver-comparison formatting,
Clippy, its two tests and run, plus the Senior benchmark-runner test. Its initial
Windows native archive overwrite failed under sandbox restrictions and succeeded
outside the sandbox. No public signatures, CI, profile policy or version data
changed in #143. The subsequent independent review found missing in-solver pause
coverage. The added test exposed a real root-timeout resume panic and passed
after `428395e` corrected it. Both review axes then reported zero open findings;
see the [review record](planning-session-review.md).
