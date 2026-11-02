# Planned regular-spin validation: issue 48, slice 1

The additive public interface and bounded delivery were explicitly approved on
2026-11-02. Existing plan/profile structs, jump entry points and scoring are
unchanged. This PR implements the first slice of #48; it does not close the issue.

## Inputs and outputs

`validate_program_with_evidence` selects the exact built-in Adult profile from
`Program`; mismatched seasons/rulesets return Unsupported. The profile-selected
`validate_profile_program_with_evidence` resolves `ProfileProgram` through the
existing `ProfileRegistry`. A custom profile needs an explicit
`RegularSpinPolicy` bound to its exact identity and a complete source citation.
No Adult spin policy is selected by category name or for a modified profile.
Explicit custom policies are caller assertions, not authenticated official rules.

`ProgramValidationEvidence.spins` associates each regular spin with one unique
zero-based element index. Wrong targets and duplicates produce input violations;
duplicate evidence is not selected arbitrarily. `RegularSpinEvidence` supplies
an intended typed abbreviation, optional target level, optional ordered feature
assertions and optional independent panel recognition. A feature target is a
zero-based position-step index; preceding ChangeFoot steps determine the foot.
Feature order within one step is the vector order. Outcomes distinguish unknown,
unsuccessful and successful attempts. None and Some(empty) remain distinct.
Free-text attempts in the legacy plan never become confirmed features.

The existing ordered `SpinStep` positions and whole revolution counts remain the
movement plan. Each position step asserts an uninterrupted planned hold. Unknown
positions/counts remain unknown; empty steps are an explicitly empty plan.
No technical-panel position or feature recognition is inferred from these inputs.
Panel recognition never fills plan gaps or changes the input; reconciliation,
including alternate recognition outcomes, is explicitly Unsupported.

`EvidenceValidationReport` contains the selected profile identity, source editions,
legacy plus additive findings and explicit family coverage. All findings retain
stable IDs, element/component indices where supported and source/page references.
Component indices in spin input findings refer to ordered position steps. Jump
component indices retain their existing semantics. Program-wide findings have no
element index. Coverage is always partial or unsupported; there is no full-valid
boolean. Successful plan checks remain conditional on execution, including the
visible flying jump, position quality, spin centres and panel identification.

## Supported-case matrix

The Adult announcement updated July 16, 2026 pp. 8–11 supplies category policy;
p. 4 gives its qualified precedence over general rules. Communication 2788,
updated May 19, 2026 p. 4 supplies additional Free Skating V conditions. The
Single Skating Technical Panel Handbook, July 15, 2026 pp. 14, 24–25 supplies
recognition cautions and multiple-foot-change scope. Official links and editions
are recorded in [rule sources](../rule-sources.md). Sources were re-read for this
slice on 2026-11-02; the downloaded Adult PDF matches the recorded SHA-256.

| Check | Supported boundary | Finding semantics |
| --- | --- | --- |
| Eligibility | Bronze non-flying one-position; Silver any second spin with a required non-flying one-position role; Gold non-flying change-foot combination and flying role; Masters non-flying combination and flying role without foot change | Prohibited intended roles are Violation; missing type is Unknown. Required alternatives share one evaluator with custom profiles. |
| Counts and abbreviations | Category maximum 2/2/3/3 spins, including ChSp; repeated typed abbreviation regardless of level | Violation for known excess/duplicates; unspecified elements can make count Unknown. ChSp recognition/mandatory presence remains slice 3. |
| Total revolutions | Bronze 3/6, Silver 4/6, Gold 4/8, Masters 5/8 without/with foot change | Known shortfall is a plan Violation, never an automatic no-value call; missing count is Unknown. |
| Basic positions | Two consecutive revolutions; combination needs two different qualifying basic positions, three for full value | Conditional no-value, V or unresolved fallback is Unknown, distinct from prohibited plans. A concluding upright wind-up does not automatically turn a one-position spin into a combination. |
| Per-foot evidence | One foot change: three revolutions each; one-position spins also need a qualifying basic position on each foot | Conditional V is Unknown. More than one change is explicitly Unsupported for per-foot checks, not prohibited. |
| Levels | Targets at or below caps 1/2/3/4; higher targets cannot receive the requested credit | Excess target is Unknown with ignored-feature explanation, not a prohibited spin. No awarded level is derived. |
| Features | Typed identity, success state, position target and order are represented; input targets are checked | Missing list is Unknown; credit, prerequisites, variation categories and program-wide reuse are Unsupported until slice 2. |
| Legacy jumps | All existing findings retained | Original partial coverage and unresolved consequences remain. |

`tests/spin_validation.rs` transcribes neighboring numeric boundaries independently
of policy construction and exercises all twenty abbreviation/entry/foot-change
combinations across all four profiles. It also covers required roles, missing
versus empty evidence, two-position versus three-position combinations,
per-foot conditions, wrong targets, ordering and a source-bound synthetic policy.
The test matrix verifies the stated subset, not official complete compliance.

## Remaining slices and limits

Slice 2 adds feature eligibility, per-foot quotas, difficult-variation categories,
first-attempt versus feature-10-first-success reuse and supported level/position
dependencies. Slice 3 adds StSq/ChSq/ChSp recognition and mandatory/category rules.
Slice 4 adds the remaining supported jump transition, Euler and global cases.
Current reports expose these coverage gaps; no scoring value logic is duplicated.
