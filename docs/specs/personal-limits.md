# Personal repertoire limits

Issue #128 adds `PersonalLimits` to planning and manual-assessment requests.
Initialize `personal_limits: PersonalLimits::default()` in existing request
literals to preserve their previous behavior, including SameDirection.
The module is available without the `optimizer` feature.

## Representation and scope

`SpinLevelLimit { abbreviation, maximum }` uses `SpinAbbreviation` to distinguish
kind, flying entry and change of foot. `Level::Basic`, `One`, `Two`, `Three` and
`Four` are the supported maxima. CCoSp is independent of CoSp and FCCoSp. Its V
variants share the same limit. Lower levels remain selectable. ChSp1, ChSq1 and
step sequences use the additive optional maximum described below.

`JumpRotationLimit { kind, maximum }` uses `JumpRotationClass::{Single, Double,
Triple, Quadruple, Quintuple}`. `TryFrom<u8>` explicitly rejects values outside
1–5. Single Axel means 1A. Limits apply at every original JumpPart position,
including combinations and sequences. Euler has no configurable maximum.

Omitted keys add no restriction. Repeated keys intersect, so the lowest maximum
wins. Personal maxima intersect category eligibility, semantic exclusions,
rotation preference and the generator's existing `PersonalSkillLimits`.
A generous personal maximum never enables a category-ineligible choice.
Use exclusions to remove a kind entirely.

## Workflow behavior

- `selectable_elements_with_limits` and `selectable_jump_elements_with_limits`
  list the permitted individual and complete atomic jump choices. The existing
  listing APIs retain their unrestricted-maxima behavior.
- Candidate generation filters automatic and supplied optional variants and
  records omissions. `CandidateGenerationAudit.personal_limits` identifies the
  repertoire used, alongside legacy skill limits and rotation preference.
- `search_maximum_bv` filters the complete theoretical repertoire before building
  its count model. `PlanningMaximum.personal_limits` identifies the proof
  context. Existing relaxed upper bounds remain admissible because their inputs
  are the filtered atoms. Required exact occurrences are never removed silently.
- Finite jump and mixed optimizers filter optional supplied variants. A mixed
  `must_have` group retains compatible alternatives; if no alternative survives
  and a personal maximum conflicts, it returns `PersonalLimitConflict`.
- Completion applies maxima to additions and reports conflicts with immutable
  originals before solving. Both locked and unlocked originals remain unchanged.
  Generated-domain reuse requires identical personal-limit provenance, just as
  for rotation preference. Different ordering or redundant entries in the limit
  vectors conservatively count as different provenance.
- `assess_manual_bv` and `assess_manual_program` preserve calculable values,
  validation and legality. `personal_conflicts` contains original element indices,
  compound component positions and typed requested/maximum values. It does not
  turn a personal preference into a competition violation.

Unknown plan attributes retain their existing missing/unsupported evidence
behavior. Limits do not infer a level or rewrite an intended jump. Invalid input,
personal fixed-input conflict, unsupported coverage and proved infeasibility
remain distinct. Restrictions are never relaxed. Cancellation and independent
equal-optimum enumeration statuses retain their existing contracts.

## Verification

The public integration tests cover Basic and levels 1–4, exact abbreviation and
V semantics, all six jump kinds and all five listed classes, compound positions,
category ceilings, exclusions, manual BV and detailed manual assessment, finite
optional filtering and required alternatives, immutable conflicts and provenance.
Independent limited-domain bounds verify Bronze 10.20 and Senior Women/Men 21.70
when all regular spins are capped at Basic and all jumps at Single. Baseline
Silver reaches 14.80, or 13.50 with Axel excluded, including optional ChSp1.


## Step-sequence maximum (#135)

The additive `selectable_elements_with_step_limit`,
`search_maximum_bv_with_step_limit` and `assess_manual_bv_with_step_limit`
entry points accept `maximum: Option<Level>` alongside existing requests and
personal limits. `None` preserves existing behavior. Basic through Four retain
all lower eligible levels and intersect category ceilings. Exact catalog values
remain unchanged; ChSq1 and ChSp1 have no invented levels.

Required over-limit steps report `PersonalLimitConflictKind::StepLevel` with the
original required-input index. Manual assessment reports the same indexed
personal conflict without changing BV or category-rule findings. Existing
request structs and entry points retain their layouts and behavior. Callers
matching every personal-conflict variant must handle the added `StepLevel`
variant. Keep the separate maximum with the request when recording provenance;
`PlanningMaximum.personal_limits` continues to describe the original jump and
regular-spin limits.
