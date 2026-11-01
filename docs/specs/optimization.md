# Optimization

## Target contract


The first objective is maximum total base value in a finite, explicitly supplied candidate domain. Each request and result records ruleset and table version, ordered element variants (including jump types, rotations, combinations, sequences and transitions), possible levels, spin positions and features, relevant jump timing, assumptions, user exclusions, equivalence definition and enumeration limits. The domain contains only modeled variants; it is not a claim to contain every physically possible program. Individual ability or exclusions narrow that domain separately from official eligibility. The selected candidates must satisfy the implemented official rules and personal constraints, including program-wide feature use and ordering. Unresolved rules prevent a claim of full official compliance or an official maximum; they must not be silently ignored.

Report separately: proven optimum within the stated model; feasible incumbent with optimality unproven; proven infeasibility; interrupted or incomplete search; and equal optimal solutions with enumeration completeness separately proven or unknown. An interruption is neither proof of optimality nor infeasibility. Revalidate every candidate against the same rule definitions and independently recompute its base value. A solver's proof applies only to its stated candidate space and implemented constraints. See the [solver decision](../adr/0001-pumpkin.md) and [comparison](../solver-comparison.md).


## Acceptance example

These are target examples. Existing tests cover some cases only in part; see [implementation status](../implementation-status.md). Other findings may coexist. See [rule sources](../rule-sources.md) for source editions. Independent human domain review is required before claiming complete correctness.

6. **Optimizer and uncertainty:** Within an explicitly complete Masters candidate domain obeying the Axel requirement, excluding all Axel types yields proven infeasibility only if the search proves it. On a small independently enumerated domain, returned optima match all distinct maxima when enumeration is proven complete. A time limit with a feasible incumbent reports unproven optimality; without one it reports incomplete, not infeasible. Unknown rules or unsupported input cannot yield a full compliance or exact-score claim.

## Exhaustive Bronze reference domain


`enumerate_bronze_reference` enumerates the eight assignments of three ordered slots at 10, 20 and 30 seconds. Each slot is empty or a solo clean 1Lz, with no calls. At least one slot must be occupied. Excluding 1Lz removes it from all slots and makes this domain infeasible. It never adds another element. Slot positions are part of candidate identity: identical jumps at different times are distinct; no other differences are modeled. The result records slot times, duration, timing and recognition assumptions, and the exclusion; each candidate records occupied slots and its program in time order.

The ruleset is the 16 July 2026 Adult announcement, Bronze Free Skating p. 11; the table is Communication 2786, 2 June 2026, p. 2, BASE column. Assume a 100-second required duration, uninterrupted timing, no applicable event-specific exception, and clean recognition of each planned jump. All starts precede 50 seconds, so this scenario uses unfactored values. These are assumptions for the model, not established facts about an actual performance. The implemented validator rejects a third 1Lz; the lookup gives 0.60 per jump. Thus the exact model maximum is 1.20 at slots (10, 20), (10, 30), and (20, 30). Enumeration checks all eight assignments, including the excluded empty assignment and the repetition violation.

The result covers only implemented Bronze jump checks and the stated domain. Spins, sequences, transitions, execution, recognition uncertainty, timing exceptions, other rules and complete program eligibility remain unverified or out of scope. This is neither full ISU compliance nor an official or global maximum. Unknown validation or base values abort the search; they do not establish infeasibility or an optimum. Source links and open applicability questions are in [rule sources](../rule-sources.md).


## Bounded Pumpkin optimizer


`optimize_bronze` accepts a subset of the three ordered reference slots, the supported ruleset and table identities, an optional 1Lz exclusion, an optional search time budget, and an equal-optimum result limit. Each enabled slot contains either no element or the clean solo 1Lz described above; occupied slot identities define distinct solutions. The non-empty requirement is part of this domain, not an official minimum. No spin, step, other jump, other category, later timing or alternative recognition is supported. The result records the domain assumptions, selected versions, enabled slots, exclusions, objective, implemented rule identifiers and source references; each candidate records its score and partial validator coverage. Source editions and implemented rules are those in the reference-domain section above.

The solver maximizes the number of 1Lz elements, equivalent to maximizing their positive table BV. Its finite feasibility relation is generated from the validator, and each returned candidate is validated and scored again. An optimization proof and complete enumeration are separate: enumeration is complete only when Pumpkin exhausts the fixed-optimum model. A result cap or timeout leaves enumeration unproven. A stopped optimization with an incumbent is feasible but unproven; interruption without an incumbent is incomplete. The time budget starts after model construction and is shared by optimization and enumeration. A zero result limit or unrepresentable deadline is rejected. This slice does not establish complete Bronze compliance, an official maximum, or applicability of the timing assumptions to a real event. Pumpkin 0.5 requires a newer Rust toolchain; WebAssembly compatibility and external proof certificates remain unverified.

The optimizer result records the requested time and solution limits and whether equal-optimum enumeration was requested. Its termination reason distinguishes an infeasibility proof, interrupted optimization, disabled enumeration, a reached solution cap, interrupted enumeration, and exhausted enumeration. An interruption reason does not assert that the time limit caused it. The time budget excludes model construction.
