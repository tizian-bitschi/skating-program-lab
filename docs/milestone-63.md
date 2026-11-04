# Product workflows (#63)

The backend supports the four conditional planning workflows in issue #63 for
the exact 2026/27 Adult Bronze, Silver, Gold and Masters Single Free Skating
profiles. The implementation baseline is `70ee0959ce27c6c94102f704fb5f09bbe41e5355`.
This parent audit connects the merged workstreams to their acceptance evidence;
it adds no public interface or rule implementation.

## Merged workstreams

| Workstream | Merged implementation | Contract and regression suite |
| --- | --- | --- |
| #64 Manual assessment | [PR #69](https://github.com/tizian-bitschi/skating-program-lab/pull/69) | [Assessment contract](specs/manual-program-assessment-proposal.md); `tests/assessment.rs` |
| #65 Candidate generation | [PR #70](https://github.com/tizian-bitschi/skating-program-lab/pull/70) | [Finite generation](specs/bounded-program-optimizer.md#finite-candidate-generation-65); `tests/candidate_generation.rs` |
| #66 Partial completion | [PR #71](https://github.com/tizian-bitschi/skating-program-lab/pull/71) | [Immutable completion](specs/bounded-program-optimizer.md#immutable-ordered-completion-66); `tests/program_completion.rs` |
| #67 Legality coverage | [PR #72](https://github.com/tizian-bitschi/skating-program-lab/pull/72) | [Coverage matrix](specs/legality-coverage-proposal.md); `tests/legality.rs` |
| #68 Workflow acceptance and scaling | [PR #73](https://github.com/tizian-bitschi/skating-program-lab/pull/73) | [Independent oracles and measured bounds](milestone-68.md); `tests/workflow_acceptance.rs` |

## Parent acceptance audit

| #63 acceptance | Executable evidence | Scope of the result |
| --- | --- | --- |
| Manual Bronze/Silver sourced BV, optional explicit GOE | `assessment::bronze_and_silver_source_checked_bv_and_missing_goe`, `absent_zero_and_mixed_goe_are_distinct`, `supplied_calls_falls_and_unsupported_outcomes_stay_separate` | Intended BV and supplied-recognition scenarios stay separate. Missing GOE leaves BV-only or incomplete scenario totals; no official deduction is invented. |
| Complete and draft legality with sources and missing requirements | `legality::full_bronze_plan_has_no_covered_violation_and_retains_open_rules`, `all_four_profiles_keep_distinct_obligations`, `gold_prohibition_survives_missing_content_and_draft_omissions_are_indexed` | `ConfirmedViolation`, `NoViolationInCoveredRules` and `Indeterminate` are bounded assessments. Per-rule coverage remains partial and source-bound; a covered pass is no full-compliance claim. |
| Generate from profile and personal filters, then optimize BV | `candidate_generation::four_profiles_match_independent_solo_oracle_in_order`, `component_filters_cover_combinations_and_sequences`; `workflow_acceptance::generated_mixed_optimum_matches_every_independently_enumerated_order_and_rechecks` | Generation supplies variants to the existing optimizer. The independent three-slot Bronze oracle proves BV 520 and all 12 ordered ties in its declared domain. Generation omissions, solver status and tie completeness remain separate. |
| Complete two locked elements, preserving order and sharing global constraints | `workflow_acceptance::generated_completion_preserves_two_anchors_and_every_oracle_tie`; `program_completion::silver_all_optimal_words_match_independent_oracle_and_assessment`, `repetitions_include_compound_components_and_spin_caps_include_originals`, `order_dependent_first_attempt_feature_is_validated_across_original_and_insert` | Whole completed programs are assessed and scored. The generated Bronze oracle checks all 24 distinct provenance-aware optima; Silver supplies a separate exhaustive completion case. Boundary/absolute-position tests cover allowed insertion locations. |
| Unresolved qualification, transitions and unsupported scope stay incomplete | `assessment::explicit_recognition_does_not_fill_missing_plan_qualification`; `candidate_generation::manual_unknown_transitions_and_skill_limits_do_not_establish_feasibility`; `legality::unevaluated_transition_retains_selected_event_replacement`; `workflow_acceptance::finite_bounds_and_identity_errors_never_produce_a_plausible_optimum` | Recognition assumptions do not supply missing evidence. Unsupported domains and identities cannot silently become Adult requests or official/global optima. |
| Preserve catalog, sources, legacy scoring and optimizers | `tests/catalog.rs`, `tests/catalog_manifest.rs`, `tests/profiles.rs`, `tests/scoring.rs`, `tests/optimizer.rs`, `tests/program_optimizer.rs` and the full feature-enabled/disabled suites | Existing exact integer arithmetic, 625-row source fixtures and compatibility tests remain the regression baseline. Source authority and full regulatory coverage require separate evidence. |

Test names above use integration-test file stems as suite names. Run the parent
checks with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` and `cargo test --no-default-features`. Solver-dependent acceptance
and completion suites are enabled by the default `optimizer` feature.

## Shared implementation and consumer obligations

`ProfileRegistry` selects exact identities and source editions. Manual assessment
and legality reuse the existing mixed-program validator and catalog scorer.
Candidate generation feeds `optimize_program`; `complete_program` uses the same
ordered Pumpkin search, whole-program constraints and manual assessment. There
is no separate product-layer validator, score table, rule DSL or solver.

Consumers must retain the generated-domain audit alongside optimization results:
available variants, omitted mechanisms, sources, assumptions and applied-rule
traces define the proof scope. `OptimalProven` proves an unfactored BV maximum
only in that finite supported conditional domain. Claim all alternatives only
when `enumeration_complete` is true. Result caps and interruptions leave that
claim unproven even when the best objective is proven. Complete candidate
generation does not establish complete regulatory coverage.

Planned elements, intended levels, execution observations, supplied panel
recognition and hypothetical GOE remain separate inputs/stages. Draft omissions
can describe repairable requirements; immutable violations and insufficient
remaining capacity still block completion. Every accepted generated or completed
program is validated as a whole and rescored from catalog values.

## Remaining limits

The [#68 measurements](milestone-68.md#measured-bounds-and-default-cap-decision)
record no incumbent in broad six-slot native domains within 250 ms. Useful
interactive performance for those domains is unproven; callers need explicit
budgets and smaller personal domains. The small exhaustive fixtures establish
correctness within their domains, without a full-length event-program or
unbounded optimization claim.

Unmodeled spin variations/prerequisites, transitions, timing/performance rules
and Adult Euler consequences remain explicit coverage limits. Execution-scoring
follow-ups [#56](https://github.com/tizian-bitschi/skating-program-lab/issues/56)
and [#58](https://github.com/tizian-bitschi/skating-program-lab/issues/58) remain
open and nonblocking for this supported subset. UI, HTTP, persistence, Pair,
Ice Dance, Short Program, researched non-Adult profiles, PCS, automatic panel
inference, official scores and WebAssembly proof remain outside this milestone.
See [implementation status](implementation-status.md) and
[source editions/open questions](rule-sources.md) for the detailed boundaries.
