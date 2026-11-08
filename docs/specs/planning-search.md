# Theoretical maximum-BV planning search

Issue #121 implements the Bronze and Silver part of #117. The public seam is
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

The Bronze/Silver theoretical rules inspect only family totals, component
repetitions, compound/sequence/three-jump/Euler totals, the coexistence of an
Euler combination with a sequence, spin abbreviation distinctness, and required
spin roles. These are functions of the multiset of whole elements. Internal
compound order and exact spin abbreviation stay in the atomic element. No
execution-feature ordering or timing bonus enters this workflow. Credited BV is
the sum of the published whole-element values in these two profiles.

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
and mandatory roles, with required exact values reserved. Independent subsection
value bounds strengthen Pumpkin propagation without removing any feasible
composition. Objective-first, descending-value count branching changes search
order only. The model checks absolute linear activity before using i32 arithmetic.

The initial model keeps every exact atomic choice. No alias reduction is
necessary for Bronze/Silver. Later category work must establish its contextual
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
has its own Exhausted, LimitReached or Cancelled status. After proving the optimum and validating its first representative, the model
fixes credited BV to the proved value and enumerates up to five distinct
whole-element multisets. A nogood excludes the complete atomic count vector,
including zero counts; auxiliary assignments and required-ID swaps never create
another composition. Exact compound codes, transition assumptions and repeated
occurrences remain part of identity. Count and value branching and reconstruction
use deterministic repertoire order.

Fewer than five alternatives carry Exhausted only after the solver proves no
further vector exists. Five alternatives carry LimitReached even when there
happen to be exactly five. Cancellation during later solving or reconstruction
retains the maximum and every already validated alternative with Cancelled.
Cancellation during initial reconstruction exposes no program.

Legacy ordered completion and evidence-based entry points retain their contracts.
