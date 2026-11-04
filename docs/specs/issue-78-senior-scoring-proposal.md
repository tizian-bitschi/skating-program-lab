# Senior protocol scoring contract (#78)

Status: public-interface contract approved by the requesting user on 2026-11-04.
Baseline: `b0945b6`, source and target `origin/main`.

## Public changes submitted for approval

Preserve existing public struct layouts, enum variants and function signatures,
including `ElementScore`, `ProgramElement`, `ManualElementScenario`,
`score_element`, `score_program`, `assess_manual_program` and
`ElementExecutionEvidence`. Their single-judge inputs remain hypothetical.

Add these separately exported types and entry points:

- `JudgePanelGrades { grades: Vec<Option<i8>> }`: ordered submitted judge
  positions; `None` means a missing grade, never zero. Valid grades are -5..=5.
  Validate panel size and missing-grade cases against Rule 353 before scoring.
- `PanelGoeScore`: submitted grades, retained judge positions, numerical GOE
  in hundredths and the applicable source references. No synthetic mean grade.
- `aggregate_panel_goe(table, recognition, annotations, grades)` returns
  `Result<PanelGoeScore, PanelScoreError>`. Resolve the shared SOV reference,
  convert grades to numerical adjustments, exclude extremes as prescribed,
  then perform the prescribed exact rounding. Do not reuse single-judge
  plausibility caps to reject individual official panel grades.
- `ProtocolElementInput`: existing `PanelRecognition`, scoring annotations,
  optional `JudgePanelGrades`, explicitly supplied printed second-half credit,
  protocol row `SourceReference`, and optional existing execution evidence.
  It has no fabricated start timestamp or hypothetical GOE field.
- `ProtocolElementScore`: separate table BV, credited BV, optional panel GOE
  and optional element total, all in hundredths; credit provenance distinguishes
  supplied official second-half credit from independently verified timing.
- `ProtocolProgramScore`: ordered per-row results and optional exact TES.
  An unresolved row prevents exact TES. No PCS or total segment score.
- `score_protocol_program(profile, table, elements)` returns the ordered replay
  report using existing profile resolution, recognition and repetition logic.
  Missing judge arrays permit BV-only results; they never imply GOE zero.
- `PanelScoreError`: invalid grade, unsupported panel configuration, invalid
  input, unresolved recognition/context and unsupported consequence, with source
  and row provenance where available. Existing error enums stay unchanged.

Use borrowed inputs for these functions. Final Rust signatures use existing
profile and table types; no new registry or rule engine is introduced.

## Shared implementation and boundaries

Expand shared catalog resolution for independently verified triple/quad call
rows. Reuse it for legacy scoring and protocol replay; do not copy SOV values.
Keep existing Adult supported behavior and fixtures unchanged.

Extend the existing Senior contextual repetition resolver for supported calls.
Distinguish two solos from a repetition in a compound. Reject unresolved or
illegal program contexts instead of converting invalid plans to legitimate zero
elements. Supplied official stars/NoValue remain explicit recognition outcomes.
An isolated supplied +REP is replay evidence, not proof of repetition eligibility.

Implementation detail: `score_protocol_program` takes the existing registry and
profile identity, like `score_profile_program`. It preserves printed REP and
deletions exactly. `score_profile_program` and manual assessment use the shared
context resolver to derive credit; replay never replaces the supplied record
with inferred calls. The replay output does not establish program eligibility.

Printed x is explicit official credit evidence. Without independent timing,
report that provenance and do not claim that the jump's start time was checked.
Do not infer falls, motion or other execution evidence from missing data.

Reuse existing manual assessment and program scoring for their existing
contracts. The additive replay report does not change their input semantics.
Keep TES, fall/time deductions, PCS and total segment scores separate.

## Evidence and validation required before completion

Verify Rule 353 panel exclusions, missing/reduced-panel applicability and signed
rounding from the official source. Verify Rule 612 repetition and Communication
2786 reference-GOE rules before adding numerical cases. Record exact locations
and a call/row coverage matrix; unsupported consequences retain explicit errors.

Replay independently transcribed #76 rows including clean triple/quad, q, <,
<<, edge calls, compounds, sequences, REP, stars and x. Compare printed BV,
numerical GOE and element totals to two decimals. Add focused tests for nine,
reduced and missing-judge configurations, tied extremes, negative adjustments,
rounding, invalid inputs and contextual repetition boundaries. Retain all Adult,
legacy hypothetical-GOE and execution-evidence regressions.

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` and `cargo test --no-default-features` after implementation.
The external review uses Chrome, GPT-6 and Sehr Hoch. Only a final APPROVE and
passed required checks permit the merge authorized by the invoked workflow.
