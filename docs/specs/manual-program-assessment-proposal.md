# Manual program assessment contract (#64)

Status: additive public interface approved by the user on 2026-11-03; implemented
contract. Exact exported field names appear in `src/assessment.rs`.

## Entry point and compatibility

Add `assess_manual_program(registry, request)` as the entry point for future UI
and adapters. Keep every existing public type and function unchanged. Put the
orchestration in a new `assessment` module, re-exported by the library. Reuse
`ProfileRegistry::resolve`, `validate_supported_program`, catalog BV lookup and
the existing execution-aware element and planning scorers. Do not duplicate SOV
data, recognition rules, spin qualification or program-wide validation.

## Request

- Exact `ProfileId` and `ScoringTableId`, a profile-bound `ProgramPolicy`, and
  `Draft` or `Complete` submission state.
- One ordered list of entries. Each entry contains its `PlannedElement`, typed
  `ElementPlanEvidence`, optional explicitly assumed recognition for planned BV,
  and an optional hypothetical scoring scenario.
- A hypothetical scenario contains supplied `PanelRecognition`, annotations,
  optional final GOE, optional fall information and optional
  `ElementExecutionEvidence`. Observations and panel decisions stay separate from
  the planned element and its intended level.
- An explicit request for a GOE-inclusive total and an optional planning-only
  second-half boundary. No timestamps, official timing or segment deductions.

Unknown rotations and unsupported spins remain representable. Missing recognition
never becomes a clean recognition. Reject identity, family and index mismatches
without silently selecting another edition. Preserve input order in every output.

All recognition stages, including an optional spin-evidence panel, reject
`observed_element: Some(_)`: this request carries no indexed observation list.
Intended and scenario recognition may omit `planned_element` because the containing
entry supplies its association; any explicit index must equal that entry's current
position. The shared spin validator retains its stricter explicit index requirement
for a panel stored inside spin evidence. Reordering entries requires updating
explicit references. Execution errors remain local component-indexed facts.

Invalid feature targets and contradictory spin entry/foot-change evidence block
qualified intended BV. Invalid or unordered feature targets receive no program-wide
feature credit and consume none. Category prohibitions and program count/repetition
violations remain separate: an identifiable prohibited Axel can retain numerical BV.
Assessment and optimizer adapters share the internal `plan_recognition` structural
encoder; numerical values and qualification remain in existing scoring/validation.

## Result

- Echo the selected profile/table, submission state and scenario assumptions.
- Return the full shared validation report, including findings, applied rules,
  element/component indices and coverage.
- Return indexed element results for intended/assumed BV and, independently,
  supplied hypothetical recognition/scoring. Retain source row, version, calls,
  execution facts and scoring failures. Planned BV and recognized BV are separate.
- Each aggregate exposes its resolved subtotal and the indices that are missing
  or unsupported. An exact aggregate is present only when every relevant element
  resolves. An empty unresolved list does not establish program legality.
- No supplied GOE anywhere means BV-only: no GOE subtotal, adjustment or combined
  total. Explicit `Some(0)` produces a real zero adjustment if supported. Mixed
  GOE produces an explicitly partial scenario with missing-GOE indices and no
  complete GOE-inclusive total. A requested combined total also requires complete
  supported recognition, execution consequences and numerical scoring.
- Unknown calls, unsupported units, ignored/no-value outcomes and missing feature
  qualification retain their existing explicit outcomes. They cannot silently
  contribute zero to an exact unsupported aggregate.
- Fall GOE consequences and any separately known deduction information remain
  distinct; the planning result does not claim an official TES or segment score.

The supplied scenario recognition is an independent explicit panel assumption.
It may differ from the intended plan (including an attempted jump with unknown
planned rotations). It is scored as supplied; it never fills missing plan
evidence. Thus a supported supplied panel level can have a numeric scenario
while the intended spin BV stays unresolved for missing feature qualification.
The validation report remains unchanged and does not become legal through that
scenario. No fall or time deductions are calculated or subtracted here; callers
use `score_profile_program` with complete event inputs for those separate results.

`planning_score` preserves credited per-element results for complete GOE scenarios
with a requested boundary. A partial GOE scenario with a boundary has no credited
subtotal: it reports every combined-score index unresolved. This avoids assigning
boundary eligibility using an invented grade. Without a boundary, supported
supplied element scores contribute to the explicitly partial combined subtotal.

Annotations containing recorded timing credit are rejected; the boundary is the
only supported timestamp-free credit assumption. `NoValue` and `Ignored` outcomes
are retained but do not count as invented zero-valued planned/scenario elements.

### Numeric fixture

Communication 2786, 2026-06-02, pp. 2, 6 and 14: 1S = 0.40,
1Lz = 0.60, 1T+1Lo = 0.40+0.50, CSpB = 1.30, ChSq1 = 3.50.
The Bronze fixture totals 6.70. Adding SSpB = 1.30 gives the Silver fixture
8.00. These are conditional intended BVs, not a proof of full rule coverage.
The tests check exact per-element source editions and maintain literal expected
values independently of production catalog lookup.

## Draft and complete assessment

Use `ConfirmedViolation`, `NoViolationInCoveredRules` and `Indeterminate` as
coverage-qualified statuses; none means fully legal. Retain all original findings.
Missing required families/roles are completion diagnostics in a draft. They
become blocking findings for a complete submission. Other established violations
remain violations in both states. Missing evidence and incomplete rule coverage
remain visible. Identify completion rules explicitly from shared validator rule
IDs, not from finding text. This is the baseline contract for follow-up #67.

## Behavioral examples and proof

1. Fixed independently source-checked Bronze and Silver fixtures exercise solos,
   a direct combination, supported spins, choreography, category obligations and
   order. Assert literal hundredths and source references.
2. The same fixture with all GOE absent returns exact planned BV and no GOE
   aggregate. Supplying zero for every supported element yields an independent
   zero-GOE scenario. Supplying GOE for one element reports the missing indices.
3. Reordering an entry reorders its score and its validator findings together.
4. Explicit q/<, edge and fall scenarios retain calls and independent outcomes;
   unknown calls and unsupported evidence prevent exact hypothetical totals.
5. An extendable draft retains missing-element diagnostics without treating those
   omissions as an irrevocable violation. Complete submission exposes them as
   blocking. Partial coverage never establishes full compliance.
6. Invalid profile/table identities fail; legacy scoring and validation remain
   compatible. Run fmt, clippy, default tests and no-default-features tests.

No frontend, HTTP API, persistence, automatic recognition, complete rule engine or
optimizer change is included.
