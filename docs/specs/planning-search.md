# Theoretical maximum-BV planning search

Issues #121–#123 implement the four Adult categories and both source-bound Senior
profiles in #117. The public seam is
`search_maximum_bv(registry, request, control)`. Required occurrences contain a
stable caller ID and an exact `AssumedRecognition`; their input order does not
constrain placement. Exclusions name jump kinds, exact listed jumps and named
regular-spin kinds. Duplicate IDs are invalid requests.

`request.rotation_preference` selects the personal connection restriction.
`RotationPreference::default()` permits same-direction compounds;
`AllowMixedDirections` explicitly requests the broader theoretical repertoire.
The result records this choice in `PlanningMaximum.rotation_preference`.
Required personal conflicts are separate errors, not competition violations or
proofs of rule infeasibility. See the [rotation contract](rotation-preference-proposal.md).

## Count model and equivalence argument

The repertoire comes from profile- and preference-filtered individual selection and complete
planned jump generation, without a caller shortlist or a variant ceiling.
Each atomic whole element has one bounded integer occurrence count. Required
occurrences impose lower bounds on their exact element, including its transition
assumption. Compounds remain indivisible.

The Adult theoretical rules inspect only family totals, component
repetitions, compound/sequence/three-jump/Euler totals, the coexistence of an
Euler combination with a sequence, spin abbreviation distinctness, and required
spin roles. These are functions of the multiset of whole elements. Internal
compound order and exact spin abbreviation stay in the atomic element. No
execution-feature ordering or timing bonus enters this workflow. Credited BV is
the sum of the published whole-element values in these four profiles.

Consequently mapping an ordered feasible program to counts preserves every
supported composition condition and its objective. Conversely, sorting the
selected exact elements reconstructs a representative with the same conditions
and BV. Assigning each required ID to a separate matching occurrence preserves
all required multiplicities. The shared manual-BV assessment checks the
representative before publication. A mismatch is an implementation error,
never a reason to report impossibility.

An admissible bound reserves required jump components and spin abbreviations,
then relaxes compound packing to the remaining profile-derived listed-jump
capacity and repetition stock. The spin bound considers distinct abbreviations
and mandatory roles, with required exact values reserved. Mandatory choreographic
spins reserve a slot in the overall spin allowance before the regular-spin bound
is calculated. Required exact step and choreographic choices consume their family
slots. Gold and Masters use the shared role predicates, including their different
foot-change restrictions. Masters retains the profile Axel obligation; an exclusion
removing every Axel produces an explicit diagnostic. Independent subsection
value bounds strengthen Pumpkin propagation without removing any feasible
composition. Exact credited-jump and regular-spin subtotal variables link these
section bounds to the overall objective, so an optimal total propagates section
limits before whole-element choices are branched. Objective-first, descending-value count branching changes search
order only. The model checks absolute linear activity before using i32 arithmetic.

The maximum-proof model keeps every exact atomic choice, including all 51,518
Senior jump choices per profile. No alias grouping is used. Shared profile policies supply
bounds and role predicates. Domain construction reuses the manual selection
resolver and caches individual component checks within one immutable profile/table
selection context; it still resolves every whole element and keeps its exact identity.

## Senior contextual credit and upper bound

Senior constraints additionally retain jump-kind totals across rotations,
identical double-or-higher totals, repeated advanced-type totals and repeated
quad/quint-type totals. Singles can occur three times, subject to the shared
kind cap. Each advanced type has exact component and solo totals. Boolean integer
flags identify total==2 for the repetition limits and solo-total==2 for credit.
The credited objective is raw whole-element BV minus the source-rounded 20% loss
for each advanced type with two solo occurrences. A compound occurrence prevents
that type's two-solo loss. The shared scoring helper supplies the rounded value.

This total loss is invariant under whole-element ordering: exactly one of two
identical advanced solos receives 80% in every reconstruction. The manual assessor
assigns that per-element credit after deterministic reconstruction. Thus counts
preserve the exact credited total while internal compound arrangements remain
separate atomic choices. The public fixture with required 5Lz and two occupied
compound slots distinguishes the raw winner (another 5Lz) from the credited
winner (4A): raw maximum50.60, credited maximum49.10.

A private Senior DP folds component vectors for each jump kind. It retains the
shared kind/identical/repeated-type caps, required component multiplicities,
Axel presence, listed-component capacity and quint solo capacity. Enumerating
jump/compound/three-jump budgets reserves the exact structure of required whole
elements; a required two-jump compound cannot become a three-jump compound.
Quints cannot occur in compounds, so their repeated-solo loss is unavoidable;
two required identical advanced solos also force their loss. Other contextual
losses and compound packing are relaxed. Every legal composition maps to one
retained state and budget with at least its credited value. The resulting bound
constrains the credited jump expression, never the raw jump sum. The solver still
proves the exact objective, and shared full manual assessment checks every
published alternative.

## Outcomes and cancellation

Invalid request, profile resolution and unsupported coverage are separate
errors. Indexed required-input diagnostics identify prohibited content and
exclusion conflicts before solving. Repairable missing roles remain completion
targets. Infeasibility requires a supported contradiction or solver proof.

Control polls identify Domain, Model, Solve, IncumbentFound, Reconstruction,
MaximumProved and Alternatives phases. A stopped proof returns Cancelled and no
program. No timeout or search-node budget is imposed. The incumbent phase reveals
only an internal candidate lifecycle state after shared reconstruction and manual
assessment have passed, never its program or value. A reconstruction error stops
the search as an error; it does not silently discard a candidate. Reconstruction
cancellation emits no incumbent notification. MaximumProved occurs after a
returned representative has passed manual assessment. Alternative collection
has its own Exhausted, LimitReached or Cancelled status. After proving the optimum and validating its first representative, the model
fixes credited BV to the proved value and enumerates up to five distinct
whole-element multisets. At fixed optimum, a sparse nogood requires at least one selected atomic count
to decrease. This is equivalent to excluding its complete count vector in the
positive-value built-in profiles: any strict supermultiset increases credited BV.
Adult credit is additive; a second identical advanced Senior solo adds its
positive 80% value, and adding a compound cannot introduce a two-solo loss.
Illegal third advanced occurrences remain forbidden. Thus no other optimum is
removed. If a nonpositive atomic value is ever present, collection falls back to
complete-vector inequalities, including zero counts. Auxiliary assignments and
required-ID swaps never create another composition. Exact compound codes, transition assumptions and repeated
occurrences remain part of identity. Count and value branching and reconstruction
use deterministic repertoire order.

Before another SAT solve, collection traverses order aliases of confirmed whole
compounds. An alias has the same combination/sequence family, Euler count, raw
value and exact listed-component multiset, and is already in the permitted domain.
Replacing one occurrence above its exact required minimum preserves every model
coefficient and the Senior solo-credit context. Full count vectors suppress
duplicates, and shared reconstruction/manual assessment confirms each candidate.
This breadth-first traversal changes collection order only; it does not group or
prune atoms. If fewer than five are found, every emitted vector is blocked before
the complete fixed-objective SAT search resumes. Cancellation keeps only already
validated alternatives.

Fewer than five alternatives carry Exhausted only after the solver proves no
further vector exists. Five alternatives carry LimitReached even when there
happen to be exactly five. Cancellation during later solving or reconstruction
retains the maximum and every already validated alternative with Cancelled.
Cancellation during initial reconstruction exposes no program.

## Conditional profile bounds after maximum proof (#130)

For every supported profile with no required occurrences, alternative collection first
checks each permitted atom against the proved maximum M. It hypothetically
requires that exact atom once and reuses the admissible total bound described
above, with the same selected repertoire, personal limits, exclusions and
rotation preference. An atom's count is fixed to zero only when this conditional
bound is strictly below M. A bound equal to M is retained.

Every optimum containing an atom is a completion of its hypothetical required
request. Its value is therefore at most that request's admissible bound. A bound
below M proves that no optimum contains the atom; pruning cannot remove an
optimal composition. The selected maximum representative must remain feasible;
an inconsistent bound is an implementation error. Compound identity, transition
assumptions, multiplicities, reconstruction and manual assessment are unchanged.

The guard is deliberately limited to requests whose required list is empty.
Requests with required occurrences keep the existing collection path. Before
calling the bound helper, atoms whose original
count upper bound is zero are skipped. Each remaining forced request contains
one individually permitted atom; the guard avoids invalid hypothetical required
multisets and their capacity arithmetic.

Profiles selecting Senior repetition rules first obtain component-presence
bounds from the credited-value DP. A component minimum is raised to at least one
without reserving a solo or a compound; optional nonquint solo losses remain
relaxed, and mandatory repeated-quint losses remain. Every legal completion
containing the component retains its DP witness. If this jump bound plus the
unchanged nonjump bound is below M, every atom containing that component can be
excluded. Surviving atoms receive the same exact forced-atom check as the other
profiles. This avoids thousands of repeated DP queries without grouping or
discarding tied exact identities.

Pruning has no category-identity whitelist. The count model exhaustively selects
its bound family from the repetition mechanism. A future enum mechanism must
make an explicit bound/model choice. New categories sharing an existing
mechanism inherit pruning once the existing coverage and policy adapters admit
their profiles; this does not automatically admit unverified categories.

This preprocessing starts after MaximumProved and polls Alternatives for each
atom. Cancellation retains the already validated maximum representative and
returns Cancelled collection status. The public API and five-alternative limit
are unchanged; five results still mean LimitReached, including when exactly
five optima exist. See the [implementation evidence](../planning-bound-pruning.md).

Legacy ordered completion and evidence-based entry points retain their contracts.


## Registered categories

`ProfileRegistry::register_with_program_policy(profile, policy)` atomically
registers a category and its explicit immutable program policy. The policy binds
to the profile identity; its family and spin citations must be included in the
profile sources. Existing `register` callers remain compatible. A custom profile
registered without a program policy still needs a supplied policy in APIs that
accept one, and cannot use the complete search or automatic selection.

Search admission depends on supported mechanisms and resolved policy, rather
than category identity. Jump rules must consistently select `JumpPlanV1` with
`AdultScoringV1`, or `SeniorJumpPlanV1` with `SeniorScoringV1`. A registration
does not implement new mechanisms. Source-edition compatibility, event override
precedence and partial legality coverage remain in force. Registered policies
cannot be replaced at individual consumer boundaries.
