# Bounded mixed Single Skating optimizer

## Immutable ordered completion (#66)

The additive `complete_program` contract was approved on 2026-11-03; see the
[request/result specification](locked-program-completion-proposal.md).
The optimizer feature gates this API. Original manual entries retain their IDs,
plans, evidence, assumptions and relative order, including entries marked unlocked.
Insertion boundaries refer to the original list, not a shifting completed index.
Absolute positions are a separate opt-in constraint. Only supplied recognition
index references are rebound to their new completed positions.

Completion extends the shared ordered Pumpkin model: each original has one
channelled position, positions increase, and forbidden gaps exclude additions.
It optimizes the full completed catalog BV and uses the existing whole-program
validator for every proposal. It reports separate existing/added/total BV and
reassesses each completion with the manual assessment API. GOE is preserved as
an optional scenario and never enters the objective. Unknown or unsupported
original recognition is an explicit error with its draft assessment; immutable
violations are a different error. Missing required draft elements can be repaired.
Unsupported requested addition definitions fail even when excluded.

Personal predicates filter additions only. Explicit global predicates also
check originals and report conflicting IDs; global variant indices are invalid.
Atomic compounds remain intact and their jump components count across the whole
program. Output identity compares ordered entries including evidence and supplied
recognition/scenarios, excluding internal anchor assignments. Identical visible
words are returned once even if an inserted element matches an original.

Proof and tie completeness cover only the supplied finite domain and supported
conditional rules. Attached generation audits retain omissions and domain caps;
they never expand proof scope. Time/proposal/output caps retain the existing
optimizer semantics. Total slots remain bounded at ten, original plus addition
definitions at 1,000, and validator proposals at 100,000. An empty request with
zero maximum total slots is rejected. Evidence requiring unsupported panel paths
remains indeterminate rather than being converted into a proven optimum.

`tests/program_completion.rs` independently enumerates the Silver 1A/SSp1 tiny
domain (24 optimal words, 3.50 BV), checks first-attempt feature order, compound
repetitions, family caps, global exclusions, absolute positions, draft repair,
unknown/illegal originals, Gold/Masters required roles, interruption and caps.

The external review found that reconstructing clean recognition for originals
lost supplied BV-changing calls. Fixed original coefficients now come directly
from the validated manual assessment, including the scorer's source references.
Candidate preparation and completed scoring consume those same coefficients;
additions continue through the supported clean-variant scorer. Regression
fixtures preserve supplied 1A< and solo +REP at .88 BV, producing 3.28 total BV
with SSp1 and two .40 additions. No call is inferred from repetition rules.

Completion results expose `assumptions`, `unmodeled_variants`, `supported_rules`
and optional `domain_completeness` directly. A capped generator can coexist with
exhaustive enumeration over its supplied variants; this never proves coverage
of the generator's full declared domain. Recognition reference traversal and
rebinding belong to the shared manual-entry domain implementation.

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

## External review corrections

The first review returned REQUEST_CHANGES. The direct-sequence assumption now
enters the existing jump validator at its sequence check; missing assumptions
and unspecified transitions remain Unknown. A Silver `1T + 1A` scenario plus
SSpB reaches the optimizer, shared validator and scorer with an independently
expected 2.80 objective and both ordered ties.

`AdultDefinition` in `profiles.rs` centrally owns the fixed 2026/27 Adult jump,
spin and mixed-family policy data. Existing constructors project this definition
into their compatible interfaces. `ProgramPolicy` binds separate step,
choreography and feature sources; evaluators remain in their domain modules.
Level conversion belongs to the shared `Level` type.

`EvidenceValidationReport.applied_rules` records successful and failed checks
at evaluator sites, with source references and element/component indices.
`ProgramOptimizationResult.supported_rules` is the available-check inventory;
its `applied_rules` is the union of checks actually executed for returned
candidates, including catalog scoring. Each candidate retains its own trace.
An available rule is not a claim that it ran on every program. Absent step or
choreographic elements do not acquire per-element recognition traces.

`max_proposals` names the actual budget: completed solver proposals checked by
the validator. It does not bound Pumpkin's internal nodes or preempt a single
solver call. Without `time_limit` there is no wall-clock bound for one solver
call. A regression uses ten candidates in ten slots (10^10 nonempty assignments)
and stops after one rejected proposal as Incomplete. An optional time limit is
shared with solver search and enumeration; bounded preparation/validation is
not individually preemptible.

Typed ChSq movements supplement legacy plan descriptions. When both are given,
they must describe the same ordered list. Supported exact descriptions are
Spiral, Arabesque, Spread Eagle, Ina Bauer, Hydroblading and Unlisted Jump
(case and separators are normalized). Conflicting or explicitly empty plans
produce a Violation. Other free prose remains Unsupported instead of being
silently treated as matching evidence. Plan None continues to represent missing
legacy descriptions, so independently supplied typed evidence remains usable.

## Finite candidate generation (#65)

The additive API was explicitly approved on 2026-11-03.
`generate_program_candidates(registry, request)` builds `ProgramVariant` values
for the existing optimizer. Exact built-in Adult 2026/27 profiles are supported;
custom profiles and other disciplines/segments reject without fallback.

The request selects jump kinds/rotations, solos, direct combinations, Axel
sequences and maximum compound length. All manual definitions are validated first and have output-capacity priority.
Automatic order is solos, combinations, sequences, Basic spins, explicit spin
templates and movement templates. Jumps use ascending rotations and T/S/Lo/F/Lz/A order. Continuations in
combinations are T/Lo; sequence continuations are Axel. Two-jump definitions
precede triples. This deliberately bounded grammar excludes Euler and other
transitions. Shared profile validation removes prohibited elements, overlong
compounds and repetitions. Masters' Axel requirement and missing spin/ChSp roles
are deferred to the whole-program validator.

`base_spins` selects abbreviations and constructs explicit eight-revolution
position holds, with both feet represented when requested. Combination spins use
Camel/Sit/Upright on each selected foot. These are Basic-level scenarios;
availability of higher SOV rows grants no level. `spin_plans` supplies explicit
positions, revolutions, level and supported qualified-feature evidence for richer
scenarios. Layback posture and difficult-variation prerequisites remain unmodeled
and receive omission reasons. `movement_plans` supplies existing typed step/ChSq/
ChSp evidence; the generator constructs their intended element definitions.
Unsupported or insufficient evidence never becomes a feasible default.

Manual variants remain available and use the same eligibility/scoring/evidence
checks; invalid manual definitions return an indexed error. Jump selections scope
the automatic jump grammar; semantic personal exclusions and skill limits apply
to all definitions, including every manual compound component. Variant-index
exclusions have no stable pre-generation meaning and reject. Empty selections
are a supported empty domain. Zero/oversized bounds and unmodeled rotation
requests reject. No timeout establishes completeness.

The audit retains exact profile/table, sources from policy and evaluated scoring/
validation, families, exclusions, full omitted definitions with reasons,
recognition/qualification assumptions, expansion counts and bounds. At most 1,000
output candidates and 100,000 attempted automatic definitions are permitted. Explicit
input templates/manual definitions total at most 1,000. A normalized semantic plan/evidence key deduplicates definitions: unordered
step feature sets are sorted/deduplicated and irrelevant direct-step flags on
solos/combinations are cleared. Ordered positions and feature attempts remain
significant. Distinct evidence scenarios remain distinct alternatives.

`CompleteDeclaredDomain` means exhaustion only of this finite grammar and these
explicit templates after documented eligibility/evidence omissions. It never
means all physical alternatives. `IncompleteCandidateCap` and
`IncompleteExpansionCap` prevent domain-wide optimality claims even when the
optimizer proves its smaller supplied universe optimal. The optimizer does not
consume this metadata automatically: callers must retain and present both scopes.

`tests/candidate_generation.rs` contains literal independent solo/compound
oracles, profile/filter/evidence/bounds/manual fixtures, movement-family checks
and direct optimizer integration retaining all four ordered equal-value ties.

### External review corrections

The first external review returned REQUEST_CHANGES. Automatic Axel sequences now
require `confirmed_sequence_transitions` to contain the exact ordered jump tuple;
`axel_sequences` alone supplies no direct-step evidence. Unconfirmed tuples are
omitted with the unsupported scoring-contract reason. This input is an explicit
hypothetical direct-step qualification, not a newly inferred physical transition.

`excluded_spin_positions` checks every position in regular spins, including both
feet of Combination spins. `excluded_spin_entries` covers both Flying and
NonFlying. Both apply to automatic/templates/manual candidates and remain in the
audit alongside semantic exclusions and explicit skill limits.

Manual definitions are all validated before automatic expansion, irrespective
of either cap. They consume output capacity first, but not the automatic
expansion budget. Every manual definition is checked even after output capacity
is exhausted; invalid input returns its original index. Valid manual definitions
omitted by the output cap retain a full omission entry. The audit separates
`manual_definitions_checked` from `attempted_definitions` (automatic only).

`prepare_variant` centrally performs scoring, local validation and source
collection for generator and optimizer. The shared validator classifies findings
as element-local, whole-program obligation or execution assumption, using policy
bindings and shared rule constants. No diagnostic text controls eligibility.
The generator omits local violations/unknowns; the optimizer retains its existing
whole-program solver semantics and rejects unresolved local input. Full programs
still pass the common validator including all deferred obligations.

Regression coverage includes exact sequence confirmation, automatic/manual
Combination-position and both-entry exclusions, capped invalid/manual priority,
normalized equivalent step sets and diagnostic-prose-independent classification.
