# Bounded mixed Single Skating optimizer

The additive interface and prerequisite validation work were approved on
2026-11-03. `optimize_program(registry, request)` handles the four current Adult
profiles and custom profiles using supported mechanisms and an explicit
profile-bound `ProgramPolicy`. Existing jump APIs remain compatible. This is
scenario planning under declared recognition assumptions, not a Technical Panel.

## Finite request and identity

`ProgramOptimizationRequest` carries exact season/profile/SOV identity, a finite
`ProgramVariant` list, typed ability exclusions, must-have alternative groups,
minimum/maximum element counts, the unfactored BV objective, ordered-variant
equivalence, a time limit, a validator-proposal limit and an output cap. Must-have
and exclusion indices refer to the original list. Variants may repeat where
shared rules allow them. Identical duplicate variant definitions are rejected.
Distinctness includes plan and evidence; two recognition/evidence scenarios for
the same movement plan are distinct. No skating timestamps are introduced.

Each variant supplies a real `PlannedElement`, local `ElementPlanEvidence` and an
explicit recognition assumption. Missing recognition is rejected. Successful
spin/step feature qualification needs a separate explicit assumption; a success
flag alone does not establish qualification. Assumptions cover local execution
quality and feature prerequisites, not program-wide eligibility or reuse. For
step feature 1, qualification includes the variety required for the target level.
Output reports always retain partial coverage and execution uncertainty.

Personal exclusions cover exact jumps, all rotations of a jump kind, spin kinds,
flying entries, typed spin features and exact variants. They are separate from
official profile eligibility. Unknown restriction forms and invalid indices are
errors. Unsupported requested variants are rejected even when also excluded.

## Supported variants and shared validation

Clean single/double solo jumps, supported two/three-jump direct combinations and
Axel sequences use existing jump rules and scoring. Sequences require an explicit
direct-step assumption. Euler and unmodeled transitions are rejected rather than
assigned invented recognition consequences. Regular spins use typed abbreviation,
ordered position/foot/revolution evidence and planned levels. All existing spin
role, count, distinctness, minimum-position and per-foot checks remain active.

`validate_supported_program` is the shared mixed-program validator. It reuses
`validate_profile_program_with_evidence` and adds family caps and ordered feature
accounting. Built-in profiles require their exact built-in program policies;
changing caps requires a separately registered profile. Custom policies are
explicit source-bound caller assertions, never authenticated official rules.

Communication 2788 (May 19, 2026), pp. 2, 4–5 supplies first-attempt feature use,
feature-10 first-success use, two features per foot for change-foot spins and the
special level-4 feature requirement. Supported qualified feature identities are
jump within spin, change of edge, both directions, increase of speed, eight
revolutions, difficult flying entry, difficult blade and windmills. Eight-revolution
evidence currently supports plain camel only; difficult-variation categories,
entrance/exit nature, jump foot-change and other missing semantics remain
Unsupported. A difficult flying feature requires a planned flying entry. Target
levels without enough eligible, unused credit remain Unknown. They cannot become
accepted optimizer scenarios. Extra features cannot exceed the profile level cap.

Adult announcement (July 16, 2026), pp. 8–11 supplies the separate family policies:
Bronze/Silver allow at most one ChSq, Gold/Masters at most one StSq and one required
ChSp alongside their required regular-spin roles. The existing total-spin caps
include ChSp. Gold steps need a skating movement, at least two clean difficult
turns, full ice coverage, a level cap of two and no credit for features 3–5.
Feature 5 needs feature 4. ChSq requires two distinct supported movement identities
and a clearly visible pattern. ChSp requires a same-foot uninterrupted hold of
at least three revolutions; counts from separate holds are never added (Q&A 1,
April 21, 2026, p. 1). Identification, music/choreographic quality and panel fallback
remain explicit recognition assumptions, not automatic calls. Source links and
precedence remain in [rule sources](../rule-sources.md).

## Proof, output and deployment limits

Pumpkin models ordered slots and catalog-value sums using the same helper as the
jump optimizer. It does not enumerate the full cross-product up front. Whole
proposals pass the shared validator; rejected assignments are blocked lazily.
Every returned proposal is validated and scored independently of solver values,
with an objective consistency check. Results retain each original variant index,
the ordered plan and evidence, each catalog BV, total BV, exact selected versions,
source inventory, rule inventory, assumptions and omitted variants.

An optimal accepted proposal proves optimality only within this finite supported
conditional model: rejected recognition/level scenarios are outside its feasible
relation. A stopped solver keeps a valid unproven incumbent or returns Incomplete.
InfeasibleProven requires exhausting the finite model. Tie enumeration is separate:
only exhaustion at the fixed optimum proves completeness; a cap, proposal budget
or interruption cannot do so. Unknown requested semantics produce an error before
search rather than a false infeasibility proof. Per-program insufficiency of a
known feature-credit rule rejects that scenario without claiming a panel call.

Bounds are at most 1,000 candidate definitions, 10 element slots and 100,000
validator proposals. Time starts before candidate preparation and includes model
construction and both search stages; validation/scoring of one bounded proposal
is not preemptible. Large domains can still reach the proposal/time cap: no
unrestricted full-program performance claim is made. Native debug fixtures check
the tiny complete domains; WebAssembly support remains unverified, and the
optimizer feature remains optional. No official/global maximum, physical
skateability, full ISU compliance, GOE/PCS or second-half credit is established.

## Verification

The optimizer tests independently enumerate finite words using literal fixture
values and independently written eligibility predicates. They compare the full
ordered tie set for the competing-feature case and permutation cases in all four
Adult profiles. Additional checks cover custom profiles, exclusions, mandatory
roles, infeasibility, zero-time interruption and output/proposal caps. Validator
fixtures cover unsuccessful first attempts, feature-10 first success, Gold step
movement/feature restrictions, ChSq distinct movements and same-foot ChSp holds.
Legacy source, scoring, jump optimizer and no-timestamp tests remain required.

Native Windows debug measurement on 2026-11-03: the five mixed optimizer fixtures, including all 120 ordered ties for a five-element Gold domain, completed in 4.95 seconds (compilation excluded). This is a tiny-domain observation, not a bound for larger requests.
