# Senior Free Skating profile contract (#77)

Status: approved by the requesting user on 2026-11-04.
Baseline: 2359f5d, source and target `origin/main`.

## Public changes proposed for approval

- Add `RulesetId::IsuSenior2026_27June12` for supplied Senior recognition.
  Preserve the Adult identity. Adding an enum variant requires downstream
  exhaustive matches to handle the new variant.
- Add `RuleMechanism::SeniorJumpPlanV1` and
  `RuleMechanism::SeniorScoringV1`. The Senior evaluator owns repetition and
  compound semantics; it does not reinterpret `JumpPlanV1` parameters.
  These additions also affect exhaustive downstream matches.
- Add `SeniorCategory::{Women, Men}`, `senior_profile`,
  `senior_program_policy`, `senior_regular_spin_policy` and
  `ProfileRegistry::senior`. Keep `ProfileRegistry::adult` unchanged.
- Use profile names `isu-senior-2026-27-women-free` and
  `isu-senior-2026-27-men-free`, version `2026-06-12`. Both select the existing
  2026/27 SOV edition; source corrections require a new profile version.
- Preserve existing public struct layouts, function signatures and Adult
  behavior. Add a separate typed Senior policy definition for constraints
  that existing `JumpPolicy` and `ProgramPolicy` cannot express. Resolve that
  definition through validated named rule bindings, never category strings.
  Synthetic profiles selecting the mechanism use the same explicit contract.

## Shared behavior and boundaries

One Senior technical definition supplies both identities. Existing profile-aware
validation, manual assessment, generation, mixed optimization and locked
completion consume the resolved policy. Catalog base values remain independent.

The Senior evaluator covers six jump elements, eleven total slots, Axel,
compound structure, Euler placement, rotation-sensitive repetition and spin,
step and choreography roles. Draft omissions remain missing obligations;
complete plans receive applicable violations. Panel recognition and planned
qualification remain separate. Missing evidence never awards a level.

Source review must establish the exact Rule 612 interactions before numerical
logic is implemented, including the second identical triple/quad solo's 80%
credit. That credit belongs to profile scoring, not the catalog. Senior time
and fall deductions remain explicitly unknown wherever the supplied facts or
supported rules do not establish a result; Adult defaults do not apply.

## Required evidence before completion

Record printed rule locations and current handbook/communication editions in
a source table and profile coverage matrix. Add independent boundary tests for
the issue's jump, element, repetition, compound, spin, transition and obligation
cases, plus custom-profile and Adult regressions. Run formatting, strict Clippy,
default tests and tests without default features. Update architecture,
requirements and implementation status.

The requesting user approved this public contract before implementation. No CI, security or
agent-instruction changes are proposed. The issue's instruction not to
self-merge remains in force.
