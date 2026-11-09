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

Required cases: Bronze solo1F; Silver exact1F+1Lo and CSSp2; remaining profiles
solo1A. Restricted cases exclude ToeLoop globally. The independent source bounds
are in [planning compliance](specs/planning-compliance.md),
[the six-profile report](milestone-117.md) and the existing bounded partition
oracles. Silver changes because the July16 announcement p10 permits a second
spin of any nature: ChSp1(350) replaces CCoSp2(300). The maxima are
jumps490 + CLSp2(290), or required CSSp2(280), +ChSp350 +ChSq350.

`tests/registered_planning.rs` separately registers a two-jump synthetic category
with explicit synthetic sources: unrestricted710, required1T690, restricted670,
combined670 hundredths. Independent manual `[1T,1T,CLSp1,ChSq1]` is compliant at670.
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

Results, revisions, commands, logs and repeated native measurements are recorded
below after the final checks. No UI implementation is included.

## Release approval

The backend changes add public enum variants, including ExactElement,
StepLevel, Waltz and AdultWaltzPlanV1. External exhaustive Rust matches need
updating. Technical acceptance and approval of this source compatibility change
are separate gates; repository-required approval remains with the parent task.

