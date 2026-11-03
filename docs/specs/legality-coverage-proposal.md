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
