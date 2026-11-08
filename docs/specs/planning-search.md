# Theoretical maximum-BV planning search

Issues #121 and #122 implement the four Adult categories in #117. The public seam is
`search_maximum_bv(registry, request, control)`. Required occurrences contain a
stable caller ID and an exact `AssumedRecognition`; their input order does not
constrain placement. Exclusions name jump kinds, exact listed jumps and named
regular-spin kinds. Duplicate IDs are invalid requests.

## Count model and equivalence argument

The repertoire comes from profile-filtered individual selection and complete
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
composition. Objective-first, descending-value count branching changes search
order only. The model checks absolute linear activity before using i32 arithmetic.

The initial model keeps every exact atomic choice. No alias reduction is
necessary for Adult profiles. Later category work must establish its contextual
credit and reduction argument separately. Shared profile policies supply the
bounds and role predicates; the search does not define another category policy.

## Outcomes and cancellation

Invalid request, profile resolution and unsupported coverage are separate
errors. Indexed required-input diagnostics identify prohibited content and
exclusion conflicts before solving. Repairable missing roles remain completion
targets. Infeasibility requires a supported contradiction or solver proof.

Control polls identify Domain, Model, Solve, IncumbentFound, Reconstruction,
MaximumProved and Alternatives phases. A stopped proof returns Cancelled and no
program. No timeout or search-node budget is imposed. The incumbent phase reveals
only a raw solver-incumbent lifecycle state, never its program or value and never
a claim that reconstruction has been checked. MaximumProved occurs after a
returned representative has passed manual assessment. Alternative collection
has its own Exhausted, LimitReached or Cancelled status. The first slice returns
one proved representative with LimitReached; #124 collects up to five distinct
whole-element multisets.

Legacy ordered completion and evidence-based entry points retain their contracts.

