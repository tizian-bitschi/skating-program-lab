# Jump rotation preference

Status: product behavior and public API changes approved by the user on
2026-11-08; implemented on `codex/rotation-preference`.

The current theoretical search may return compounds requiring opposite-direction
jumps. Make same-direction compounds the default across supported categories;
explicit opt-in removes this personal restriction. Separate solos need not share
a direction. This is a planning preference, not a change to competition rules.

Product decisions: [discussion](rotation-preference-discussion.md).
Runnable primary source: `src/planning_search/rotation-preference.prototype.html`,
branch `codex/prototype-rotation-preference`, prototype commit `438e748`.

## Source basis and derivation

- [ISU Sports Rules, effective 12 June 2026, Rule 610, printed p. 65](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Isu-Regulations/2026-FS---Pair-Sports-Rules-FINAL-20260513-1778685370-9916.pdf#page=65):
  combination connections preserve landing/takeoff foot; sequences allow a direct
  step into a subsequent Axel. An Euler exit cannot change feet before the next jump.
- [Technical Panel Handbook, 15 July 2026, printed pp. 28–29](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Sports-Rules/Figure-Skating-Rules/Handbooks-Single-%26-Pair-Skating/TP-HandbookSingles-26-27-15-July-2026-1784641784-3364.pdf#page=28):
  an intended three turn, rocker, counter or bracket can preserve a combination.
  It explicitly includes listed jump + Euler + same-foot three turn + Axel.
  An intended edge change is permitted between jumps in different rotational directions.
- [ISU jump descriptions, 29 July 2026](https://www.isu-skating.com/en/news/figure-skating-jumps-explained/):
  takeoff edges distinguish jump kinds. Axel takes off forward outside; Salchow
  and Flip backward inside; Lutz, Loop and Toe Loop backward outside.
- [U.S. Figure Skating 2019–20 Media Guide, Euler definition](https://www.usfigureskating.org/sites/default/files/2019-12/2019-20%20U.S.%20Figure%20Skating%20Media%20Guide.pdf):
  Euler takes off backward outside and lands on the opposite foot backward inside.
  Used only for movement mechanics, not current scoring or competition limits.

The tables below are an engineering derivation from those definitions, not an
ISU-published whitelist. Assume standard listed-jump takeoffs and landings. Do not
infer exceptional one-foot landing variants or skater ability from a code.
The filter is a complete classifier for the existing clean code/connection model,
not a simulation of every possible performed movement.

## Same-direction transition table

Use counterclockwise jumps as the reference. Clockwise execution mirrors left
and right, without changing which connections pass.

| Jump kind | Reference takeoff | Reference landing |
| --- | --- | --- |
| T | right backward outside + left toe pick | right backward outside |
| Lo | right backward outside | right backward outside |
| S | left backward inside | right backward outside |
| F | left backward inside + right toe pick | right backward outside |
| Lz | left backward outside + right toe pick | right backward outside |
| A | left forward outside | right backward outside |
| Eu | right backward outside | left backward inside |

| Connection context | Permitted next listed kinds in same direction | Basis |
| --- | --- | --- |
| After a listed jump, without Euler or sequence step | T, Lo | Matching foot and backward outside edge |
| After a listed jump followed by Euler | S, F | Matching left backward inside takeoff |
| After a listed jump followed by Euler and one same-foot three turn | A | Left backward inside turns to left forward outside |
| Direct step from a listed jump into an Axel, in a sequence | A | Right landing foot steps to left forward outside |

No arbitrary change of foot or extra connecting turn is invented to make a code
pass. The stated same-direction rules do not add a standalone edge change.
Rotations do not change connection classification. Profile eligibility,
repetitions, compound length, Euler count and other composition checks remain
independent.

### Whole-element evaluation

1. A solo has no within-element direction restriction.
2. For a combination, check every listed-jump gap using the applicable row above.
3. For a sequence, check every gap and require at least one direct-step Axel gap.
   Without Euler, a following Axel supplies that step. Other gaps must satisfy
   combination connections. Euler + Axel uses the same-foot turn and supplies
   no direct step; a different gap must supply the sequence step.
4. Check the whole element, not only one favorable pair. `+SEQ` does not exempt
   the other connections from the preference.
5. With mixed directions enabled, remove this personal filter. Existing rule,
   recognition and supported-domain checks still apply. Opt-in is not proof that
   every theoretical code has a physically demonstrated execution.

No new "unresolved connection" product state. Incomplete typed plans or explicit
unmodeled transitions retain their existing unsupported/evidence treatment; they
must not be mislabeled as mixed-direction connections.

## Public API

Introduce one shared type, available without the optimizer feature:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationPreference {
    #[default]
    SameDirection,
    AllowMixedDirections,
}
```

Add `pub rotation_preference: RotationPreference` to these existing requests:

- `PlanningSearchRequest`
- `CandidateGenerationRequest`
- `JumpOptimizerRequest`, `ProfileJumpOptimizerRequest`
- `ProgramOptimizationRequest`, `ProgramCompletionRequest`
- `ManualBvRequest`, `ManualProgramRequest`

Change the selection signature to:

```rust
pub fn selectable_jump_elements(
    registry: &ProfileRegistry,
    profile: &ProfileId,
    table: ScoringTableId,
    rotation_preference: RotationPreference,
) -> Result<Vec<AssumedRecognition>, ProfileSelectionError>;
```

These are intentional source-breaking changes. Update every internal caller,
example, benchmark and test. Callers wanting the ordinary setting pass
`RotationPreference::default()`; theoretical broad-domain fixtures explicitly
pass `AllowMixedDirections`. Do not retain an unfiltered legacy default.
No new property is added to competition rule profiles, `ListedJump`, scoring
tables or panel-recognition inputs.

### Personal conflicts

Introduce a common diagnostic separate from `Finding` and ISU rule coverage:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationPreferenceConflict {
    pub element_index: usize,
    pub component_indices: Vec<usize>,
}
```

Indices refer to the original input element and its `JumpPart` positions,
including Euler positions. For multiple offending gaps, include the sorted unique
indices of all involved listed jumps and transitions. This lets callers render
localized explanations and resolve original IDs without opaque English strings.

- Add `rotation_conflicts: Vec<RotationPreferenceConflict>` to
  `ManualBvAssessment` and `ManualProgramAssessment`. Calculate BV and existing
  legality independently; these conflicts cannot invalidate an otherwise
  calculable total or create a competition violation.
- Add `RotationPreferenceConflict(Vec<RotationPreferenceConflict>)` variants to
  `PlanningSearchError`, `ProgramOptimizationError` and `ProgramCompletionError`. Required search elements and
  completion originals are checked before solving, in input order. Do not call
  a preference conflict rule infeasibility or silently remove the requirement.
  Finite optimization reports a conflict when every alternative in a required
  group conflicts; otherwise the compatible alternatives remain eligible.
- Completion retains every original regardless of its `locked` flag: that flag
  does not authorize the solver to rewrite an original. Resolve conflicts by
  changing the request explicitly.
- Generation and finite optimization filter optional variants, including
  caller-supplied variants, with the same predicate. Generation audit records
  omissions as personal restrictions. All required/anchored entries are checked
  before optional filtering, so they cannot disappear.
- Add `rotation_preference` to `CandidateGenerationAudit` and reject a generated
  domain attached to completion under a different preference. Pass the setting
  through profile adapters, assumed-recognition paths and completion's internal
  optimization request.

Use one internal typed connection classifier. The generator, manual diagnostics,
search and completion must not maintain separate string-based whitelists.
Preserve input identity and scoring/recognition assumptions during filtering.
Numerical scoring and rule-only validators do not acquire personal restrictions.
Fixed historical Bronze examples with no compounds need no new controls.
`PlanningMaximum.rotation_preference` records which repertoire its proof covers.

### Adult upper bound under the standard preference

All S/F/Lz/A components need an element's first-jump place unless connected by
Euler or a sequence step/turn. Beyond the first place, an Euler combination adds
at most one such component. A sequence adds no more than its remaining listed
places and cannot coexist with a separate Euler combination. Therefore their
total is bounded by the jump-element limit plus the larger of the Euler budget
and the sequence's extra listed-place budget. Reserve required components, then
maximize remaining component value subject to that cap and existing repetition
and listed-place caps. This is a relaxation: it may retain unpackable choices,
but cannot exclude a same-direction program. Senior keeps its separate bound.

## Acceptance cases

| Code | SameDirection | AllowMixedDirections |
| --- | --- | --- |
| `1Lz+1T` | available | available |
| `1F+1Lo` | available | available |
| `1Lz+1Lz` | filtered / personal conflict if fixed | existing broad-domain behavior |
| `1F+1Lz` | filtered / personal conflict if fixed | existing broad-domain behavior |
| `1A+1A+1Lz` | filtered / personal conflict if fixed | existing broad-domain behavior |
| `1Lz+1A+1A+SEQ` | available | available |
| `1Lz+1A+1T+SEQ` | available | available |
| `1Lz+Eu+1F` | available | available |
| `1Lz+Eu+1A` | available as combination | existing broad-domain behavior |
| `1Lz+Eu+1A+SEQ` | no qualifying sequence step; existing structural rejection | same structural rejection |
| `1Lz+Eu+1F+1A+SEQ` | direction-compatible where profile permits shape | existing broad-domain behavior |
| `1Lz+1Lz+1A+SEQ` | filtered: the first gap still fails | existing broad-domain behavior |

Verify the same classification for all rotations and mirrored directions;
category restrictions remain authoritative. Enabling mixed directions must not
remove a default-domain choice.

Re-run the original unconstrained Silver search in both modes. Under the standard
setting, the source-valued witness `1Lz+1A+1A+SEQ`, `1F+1Lo`, `1Lz`, `1F`,
`CCoSp2`, `CLSp2`, `ChSq1` totals 14.30. The old theoretical upper bound also
equals 14.30; restricting that domain cannot increase it. This establishes the
expected optimum algebraically; integration must still reproduce it and return
only preference-compatible alternatives. The order of equal optima may change.

Verify manual BV still reports the original 14.30 for the original program while
separately identifying its conflicting compound elements. Fixed conflicts must
stop search/completion before a solver proposal. Explicit opt-in must clear only
the personal conflict, without changing unrelated rule findings or supported-domain
limits.

## Implementation and validation

Implement shared classification and manual separation first, then selection and
search, generation/finite optimizers and completion. Use regression cases at
the actual public entry points, including mismatched generation provenance,
immutable inputs, required multiplicities, cancellation and full-element checks.
Update domain fixture expectations independently from source-derived transitions,
not by copying the generator output.

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` and `cargo test --no-default-features`. Re-run affected planning
benchmarks when the changed domain makes existing expectations obsolete.
Do not merge the HTML shell into production. Do not broaden the detailed evidence
validator or implement explicit per-jump foot/direction observations as part of
this preference change.
