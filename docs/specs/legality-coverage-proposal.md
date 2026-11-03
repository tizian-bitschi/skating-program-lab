# Legality coverage contract and source matrix (#67)

Status: additive public interface approved on 2026-11-03. Existing public types
and signatures stay unchanged. `assess_program_legality` returns the shared
`EvidenceValidationReport`, submission state, per-rule coverage and indices of
repairable draft absences. The legacy findings remain final-check findings;
consumers must apply `repairable_absences` when presenting a draft. Unknown
qualification is never a definite repairable absence.

## Reading the report

`Evaluated` means one rule was checked, including any violation. `Conditional`
means qualification/transition assumptions or a repairable draft absence remain.
`MissingEvidence` identifies absent facts; `Unsupported` identifies an unmodeled
rule or recognition mechanism; `NotApplicable` identifies a requirement or family
absent from this selected plan. Every entry carries its source edition/location,
reason, element/component index where applicable, and the exact selected event
replacement when present. An applied-rule trace alone is not a coverage inventory.

No status proves full ISU legality. In particular, actual timing and panel
recognition remain unevaluated even for a complete plan with no covered violation.
Scoring outcomes, calls, falls and numerical GOE do not rewrite intended legality.
A prohibited Gold 2F remains a violation even if another rule lacks evidence.

## Source audit

Rechecked 2026-11-03 against the official [handbook index](../rule-sources.md).
The July 16 Adult announcement, May 19 Communication 2788, July 15 Single
Technical Panel Handbook and Q&As 1/2/3 remain the indexed editions used here.
The official handbook PDF timed out; its federation-hosted July 15 copy was
reviewed. Precise citations and official URLs are in [rule sources](../rule-sources.md).
The June 2 Communication 2786 supplies scoring values, not category permission.
Adult p. 4 gives qualified announcement precedence: check special Adult comments
in a Communication before resolving a conflict. Event replacements remain explicit
profile overrides, with original identity, justification and scope preserved.
An unresolved regulatory interpretation is not an evaluated pass.

## Built-in profile matrix

Sources: Adult July 16 pp. 8 (Masters), 9 (Gold), 10 (Silver), 11 (Bronze).
These are planned technical-content limits, separate from recognition effects.

| Rule family | Bronze | Silver | Gold | Masters |
| --- | --- | --- | --- | --- |
| Jump eligibility and element cap | singles except Axel; 4 | singles; 4 | singles, 2T/2S/2Lo; 5 | singles/doubles; 5; Axel required |
| Compounds, Euler and repetitions | one two-jump combination; no Euler | two compounds; at most one sequence and one three-jump unit; Euler once | same as Silver | same as Silver |
| Spin abbreviation/count/role, including ChSp | two one-position, non-flying | two; non-flying one-position required | three; non-flying change-foot combination, flying spin, ChSp required | three; non-flying combination, no-change-foot flying spin, ChSp required |
| Total spin revolutions, no/with foot change | 3/6 | 4/6 | 4/8 | 5/8 |
| Regular spin / step level cap | 1 / no StSq | 2 / no StSq | 3 / 2 | 4 / 4 |
| Sequence family | at most one ChSq | at most one ChSq | at most one StSq; skating movement required; features 3-5 excluded | at most one StSq |
| Duration (actual timing absent) | 1:40 +/-10s | 2:00 +/-10s | 2:50 +/-10s | 3:00 +/-10s |

All listed-jump occurrences, including compound components, share the two-occurrence
limit. A combination accompanying a sequence cannot contain Euler. Drafts may
insert missing obligatory content; prohibited content, excess counts and repeated
components remain violations. Complete submissions retain obligatory-content
violations. Missing abbreviation, position, qualification or direct step remains
unknown in both stages. No national/Junior profile is selected by name: registration
and explicit policies/mechanisms are required; unsupported mechanisms are rejected.

## Rule inventory and proof boundaries

The report includes all executed `AppliedRule` entries, plus unevaluated family,
profile and Adult-only gaps. IDs below are stable; category-specific jump IDs
come from the selected `RuleBinding`, including event replacements.

| IDs / family | Source location | Checked or deferred |
| --- | --- | --- |
| selected Input, Eligibility, Count, CompoundCount, Structure, Euler, Repetition, Axel, Transition bindings | Adult category page above; Sports Rules June 12 Rule 610 p. 65; input contract for malformed rotations | Shared structural jump checks, counts and required families. Unknown identities/steps remain unknown. |
| `program-sequence-direct-step` | 2788 May 19 p. 3; Adult category page | Conditional on explicit direct-step assumption; otherwise missing evidence. |
| `spin-count`, `spin-distinct-abbreviation`, `spin-required-role`, `spin-eligibility`, `spin-abbreviation`, `spin-entry`, `spin-level-cap`, `spin-minimum-revolutions` | Adult category page | Explicit profile roles, totals and caps; a level cap is a recognition limit, not a prohibition of the whole spin. |
| `spin-position-evidence`, `spin-basic-position`, `spin-recognition-minimum`, `spin-position-type`, `spin-combination-positions`, `spin-layback-position` | Single Handbook July 15 pp. 14, 18 | Supplied positions and holds only; two-position/V and no-value recognition remain distinct from prohibited choreography. No Layback shape is inferred from Upright. |
| `spin-per-foot-revolutions`, `spin-multiple-foot-changes`, `spin-position-policy`, `spin-per-foot-policy` | 2788 p. 4; explicit profile policy | Same-foot segment checks; absent policy or unsupported structure stays explicit. |
| `spin-evidence-target`, `spin-evidence-entry`, `spin-evidence-foot-change`, `spin-wind-up-target`, `spin-feature-target`, `spin-feature-evidence`, `spin-rotations` | input contract; cited Adult/Handbook policy | Input consistency and missing facts. Invalid targets receive no credit. |
| `spin-execution-recognition`, `spin-panel-target`, `spin-panel-reconciliation` | input contract; Handbook pp. 2, 14, 18 | Actual execution and supplied panel evidence remain separate; a plan does not establish panel recognition. |
| `program-feature-once-per-program`, `program-feature-per-foot`, `program-spin-level-credit`, `program-spin-level-four-feature` | 2788 pp. 2, 4 | Ordered bounded credit and per-foot quota. Feature 10 waits for first established success; unknown prior success blocks certainty. Other supported features consume their first valid-target attempt. |
| `program-feature-qualification`, `program-feature-position`, `program-flying-feature-entry`, `program-eight-position`, `program-eight-revolutions` | 2788 pp. 2, 4-5 | Supported features require explicit local qualification. Position prerequisites are checked; eight-revolution subset is plain camel. Edge direction, uninterrupted holds, blade control, windmill angle/count require richer evidence and remain assumptions. |
| `program-variation-categories` | 2788 pp. 2, 5; Q&A 3 October p. 1 Q4 | Unsupported: typed variation categories, category reuse, two-variation quota and features performed with used variation positions. Generic `DifficultVariation` assertions cannot earn credit. |
| `program-step-recognition`, `program-step-feature-dependency`, `program-step-level-credit` | Adult category page; 2788 pp. 2, 5; Q&A 2 May p. 1 | Supplied clean-turn count, coverage, Gold movement/caps, qualified feature identities and feature 5 dependency; actual edges/turn varieties require qualification assumptions. |
| `program-choreographic-sequence`, `program-choreographic-plan-consistency` | 2788 p. 6; Adult category page; input contract | Bounded typed movements and visibility; exact vocabulary reconciliation. Musical fit/panel boundary identification remains missing. |
| `program-choreographic-spin`, `program-required-choreographic-spin` | Q&A 1 April p. 1 Q1; Adult pp. 8-9 | Three consecutive revolutions on one foot; split-foot holds are not summed. Fallback identification remains a panel decision. |
| `program-family-cap` | explicit step/choreographic policy, Adult category page | Evaluated separately for StSq, ChSq and ChSp, preserving each source. |
| `program-duration`, `program-performance-panel`, `program-waltz-recognition` | Adult category page / p. 6; 2788 pp. 2, 4-6; Handbook pp. 2, 25, 30-33 | Missing actual timing/performance; unsupported Waltz representation and counting consequences. |

The legacy standalone spin validator retains its deferred `spin-feature-credit`
result; mixed-program validation evaluates supported features across program order.
Euler no-foot-change evidence and automatic subsequent `*`/`+REP` consequences
(Handbook p. 33) remain unresolved; existing indexed Euler unknown findings expose
this limit. ChSq content is not counted as independent listed jump elements.
Other unsupported movement types remain outside the bounded vocabulary.

## Validation and open follow-ups

`tests/legality.rs` independently exercises draft/final obligations, Gold 2F,
full Bronze content with remaining coverage, sequence evidence, component repetition,
first-success ambiguity, upright feature prerequisites and a synthetic event override.
Existing source fixtures in `program_validation`, `spin_validation`, `validation`,
`assessment`, `candidate_generation`, `program_optimizer` and `program_completion`
cover neighboring rules and shared consumers. Run both feature configurations,
formatting and clippy before merge.

#56 (touch-down/weight-transfer numerical GOE) and #58 (multiple full-revolution
numerical GOE) remain open nonblocking scoring follow-ups. They do not turn an
unsupported numerical outcome into an exact score. Full difficult-variation
recognition requires a separate typed evidence contract; no panel outcome is
inferred from descriptive choreography.

## Per-rule, per-profile implementation matrix

This matrix describes the supported mechanism, not the outcome of a particular
program. Runtime applicability also depends on the submitted element/feature.
`I`: implemented structural check; `C`: implemented subject to supplied evidence
or recognition/qualification assumptions; `U`: unsupported mechanism; `N`: not
applicable to that profile. `I/U` and `C/U` identify the bounded subsets described
in the inventory above. A missing input changes an instance to MissingEvidence;
a violation remains a separate evaluated result. Several entries can share a
rule/source/index: every finding is retained, including simultaneous Violation,
Unknown and Unsupported. Consumers must never interpret one missing-evidence
entry as replacing the other results for that instance.

Source keys retain the editions in rule-sources.md:

- A: Adult announcement **2026-07-16**, Bronze p. 11, Silver p. 10, Gold p. 9,
  Masters p. 8, section 4 Free Skating; precedence p. 4.
- L2/L3/L4/L5/L6: Communication 2788 **2026-05-19**, printed pages 2/3/4/5/6.
- H14/H18/H33: Single Technical Panel Handbook **2026-07-15**, printed pages
  14 (positions/wind-up), 18 (combination recognition), 33 (Euler consequences).
- S65: Sports Rules **2026-06-12**, p. 65 Rule 610 (jump combinations).
- Q1: Committee Q&A 1 **2026-04-21**, p. 1 Choreospins Q1.
- Q2: Committee Q&A 2 **2026-05-19**, p. 1 step feature 5.
- Q3: Committee Q&A 3 **2026-10-08**, p. 1 Q4 difficult camel recognition.
- IN: Skating Program Lab input contract, issue-39 (listed-jump rotation),
  issue-48-slice-1 (indexed spin evidence), issue-49 (typed choreography reconciliation).
  These validate input representation; they are not regulatory authority.

Fixtures: V = tests/validation.rs; SV = tests/spin_validation.rs;
PV = tests/program_validation.rs; L = tests/legality.rs; A64 = tests/assessment.rs.
Each file contains independent neighboring cases. The new L regression fixtures
are named explicitly after this table. Source-derived numerical fixtures remain
in tests/scoring.rs and tests/execution_scoring.rs and do not establish plan permission.

| Stable rule ID | Bronze | Silver | Gold | Masters | Exact source key/location | Fixtures |
| --- | --- | --- | --- | --- | --- | --- |
| listed-jump-invalid-rotations | I | I | I | I | IN issue-39, ListedJump rotation count | V |
| isu-adult-2026-27-bronze-jump-eligibility | I | N | N | N | A p. 11 jumps | V, L |
| isu-adult-2026-27-silver-jump-eligibility | N | I | N | N | A p. 10 jumps | V, A64 |
| isu-adult-2026-27-gold-jump-eligibility | N | N | I | N | A p. 9 jumps | V, L |
| isu-adult-2026-27-masters-jump-eligibility | N | N | N | I | A p. 8 jumps | V |
| isu-adult-2026-27-bronze-jump-element-count | I | N | N | N | A p. 11 a | V |
| isu-adult-2026-27-jump-element-count | N | I | I | I | A pp. 8-10 a | V |
| isu-adult-2026-27-bronze-combination-count | I | N | N | N | A p. 11 a | V |
| isu-adult-2026-27-combination-count | N | I | I | I | A pp. 8-10 a | V |
| isu-adult-2026-27-bronze-combination-structure | I | N | N | N | A p. 11 a | V |
| isu-adult-2026-27-combination-structure | N | C | C | C | A pp. 8-10 a, direct Axel step | V, PV, L |
| isu-adult-2026-27-bronze-combination-euler | I | N | N | N | A p. 11 a, Euler prohibition | V |
| isu-adult-2026-27-combination-euler | N | C/U | C/U | C/U | A pp. 8-10 a; H33, automatic panel consequence deferred | V, PV |
| isu-adult-2026-27-bronze-jump-repetition | I | N | N | N | A p. 11 a, listed-jump occurrences | V |
| isu-adult-2026-27-jump-repetition | N | I | I | I | A pp. 8-10 a, listed-jump occurrences | V, L |
| isu-adult-2026-27-masters-axel-required | N | N | N | I | A p. 8 a | V, L |
| isu-sports-2026-jump-transition | C | C | C | C | S65 Rule 610 | V, PV |
| program-sequence-direct-step | N | C | C | C | L3 jump sequences; A pp. 8-10 a | PV, L |
| spin-count | I | I | I | I | A category page b, including ChSp | SV |
| spin-required-role | N | I | I | I | A category page b | SV, A64, L |
| spin-distinct-abbreviation | I | I | I | I | A category page b | SV |
| spin-abbreviation | I | I | I | I | A category page b | SV |
| spin-eligibility | I | I | I | I | A category page b | SV |
| spin-entry | I | I | I | I | A category page b | SV |
| spin-level-cap | C | C | C | C | A category page, technical-element level cap | SV |
| spin-minimum-revolutions | I | I | I | I | A category page b, minimum revolutions | SV |
| spin-rotations | C | C | C | C | A category page b, unspecified holds | SV |
| spin-position-policy | U* | U* | U* | U* | IN issue-48, explicit source-bound policy requirement | SV |
| spin-position-evidence | C | C | C | C | H14/H18, consecutive basic-position holds | SV |
| spin-basic-position | C | C | C | C | H14, basic-position recognition | SV |
| spin-recognition-minimum | C | C | C | C | H14, minimum spin revolutions | SV |
| spin-position-type | C | C | C | C | H14, only one-position types; N for combination instances | SV, L |
| spin-combination-positions | N | C | C | C | H18, two-position/full-value requirements | SV, L |
| spin-layback-position | C | C | C | C | H14, posture not represented by Upright alone | SV |
| spin-per-foot-policy | U* | U* | U* | U* | IN issue-48, explicit per-foot policy requirement | SV |
| spin-per-foot-revolutions | C | C | C | C | L4, change-foot holds; A category page b | SV |
| spin-multiple-foot-changes | U | U | U | U | L4, unsupported multi-change-foot evidence | SV |
| spin-evidence-target | I | I | I | I | IN issue-48, one correctly indexed spin entry | SV |
| spin-evidence-entry | I | I | I | I | IN issue-48, abbreviation/entry consistency | SV |
| spin-evidence-foot-change | I | I | I | I | IN issue-48, abbreviation/ordered changes | SV |
| spin-wind-up-target | I | I | I | I | IN issue-48, explicit final upright wind-up target | SV |
| spin-feature-target | I | I | I | I | IN issue-48, position-step and attempt order | SV, A64 |
| spin-feature-evidence | C | C | C | C | IN issue-48, absent versus empty assertions | SV |
| spin-feature-credit | U* | U* | U* | U* | IN issue-48, standalone feature evaluator deferred | SV, PV |
| spin-execution-recognition | C | C | C | C | IN issue-48; H14/H18, plan/actual execution boundary | SV, A64 |
| spin-panel-target | I | I | I | I | IN issue-48, independently supplied panel association | SV, A64 |
| spin-panel-reconciliation | U | U | U | U | IN issue-48, actual panel reconciliation deferred | SV, A64 |
| program-feature-once-per-program | C | C | C | C | L2, first attempt / first successful spin | PV, L |
| program-feature-per-foot | C | C | C | C | L2, at most two credited features per foot | PV, L |
| program-feature-qualification | C/U | C/U | C/U | C/U | L2/L4/L5, supported subset and explicit qualification | PV, L |
| program-feature-position | I/U | I/U | I/U | I/U | L2 features 7-9/12; L5 blade/speed positions | L |
| program-flying-feature-entry | N | I | I | I | L2 feature 11, flying entry prerequisite; A category page b | PV |
| program-eight-position | I/U | I/U | I/U | I/U | L2 feature 10, bounded plain-camel subset | PV |
| program-eight-revolutions | C | C | C | C | L2 feature 10, uninterrupted same-foot hold | PV, L |
| program-spin-level-credit | C | C | C | C | L2, level feature counts | PV, L |
| program-spin-level-four-feature | N | N | N | C | L4, one mandatory level-four feature | PV |
| program-step-recognition | N | N | C | C | A pp. 8-9 c; L2/L5 clean turns/full coverage | PV |
| program-step-feature-dependency | N | N | C | C | L5; Q2 p. 1 feature 5 depends on feature 4 | PV |
| program-step-level-credit | N | N | C | C | A pp. 8-9 level cap/Gold movement; L2 | PV |
| program-choreographic-sequence | C | C | N | N | A pp. 10-11 c; L6 two distinct visible movements | PV, L |
| program-choreographic-plan-consistency | I | I | N | N | IN issue-49, ordered exact typed vocabulary | PV |
| program-choreographic-spin | N | N | C | C | Q1 p. 1 Q1, consecutive same-foot hold | PV |
| program-required-choreographic-spin | N | N | I | I | A pp. 8-9 b | PV, L |
| program-family-cap | I | I | I | I | A category page b/c, explicit family caps including zero | PV |
| program-duration | C | C | C | C | A category page duration; A p. 6 deductions | L |
| program-performance-panel | C | C | C | C | L2/L4-L6 actual edges/holds/identification; H14/H18 | L |
| program-waltz-recognition | U | U | U | U | A pp. 9-11 Waltz treatment; Masters scope unresolved | L |
| program-variation-categories | U | U | U | U | L2 category quotas/reuse; L5 used-position feature; Q3 Q4 | PV, L |

`U*` is an inactive guard for built-in mixed-program checks: the built-in policies
supply position/per-foot mechanisms and use program-wide feature evaluation.
Standalone or custom calls without those mechanisms return Unsupported rather
than inheriting Adult rules. N describes regulatory nonapplicability; a supplied
prohibited element can still produce a successful prerequisite check alongside
its separate eligibility violation. For example, a prohibited Bronze flying spin
can trigger a flying-entry diagnostic without making that family permissible.

Review regression tests in L:

- `missing_local_qualification_preserves_earlier_possible_eight_revolution_success`
  and `first_success_is_spin_scoped_and_can_use_the_second_foot`: L2 feature 10.
- `coverage_retains_violation_and_unknown_for_the_same_rule_instance`: A p. 11,
  a prohibited Axel remains visible despite missing rotations.
- `successful_position_prerequisites_are_traced_and_combination_type_is_not_applicable`:
  L2/L5, H14/H18 applicability and successful trace entries.
- `competing_required_jump_families_are_not_jointly_insertable_in_one_slot`:
  synthetic explicit family/count policy, including registry rejection of the
  original contradictory max-one profile and a reachable max-two neighbor.
- `unevaluated_transition_retains_selected_event_replacement`: synthetic scoped
  Transition binding, exact override metadata even without compound elements.

Draft repairability is a shared capacity check, not a proof of an optimizer
completion within a supplied finite domain. Missing families compete jointly for
jump, compound, spin and choreography slots. A required Axel can share a required
jump-family insertion; a regular-spin insertion cannot serve as a ChSp. Manual
assessment and completion-original assessment use this same shared calculation.
