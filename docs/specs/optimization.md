# Optimization

## Target contract


The first objective is maximum total base value in a finite, explicitly supplied candidate domain. Each request and result records ruleset and table version, ordered element variants (including jump types, rotations, combinations, sequences and transitions), possible levels, spin positions and features, relevant jump timing, assumptions, user exclusions, equivalence definition and enumeration limits. The domain contains only modeled variants; it is not a claim to contain every physically possible program. Individual ability or exclusions narrow that domain separately from official eligibility. The selected candidates must satisfy the implemented official rules and personal constraints, including program-wide feature use and ordering. Unresolved rules prevent a claim of full official compliance or an official maximum; they must not be silently ignored.

Report separately: proven optimum within the stated model; feasible incumbent with optimality unproven; proven infeasibility; interrupted or incomplete search; and equal optimal solutions with enumeration completeness separately proven or unknown. An interruption is neither proof of optimality nor infeasibility. Revalidate every candidate against the same rule definitions and independently recompute its base value. A solver's proof applies only to its stated candidate space and implemented constraints. See the [solver decision](../adr/0001-pumpkin.md) and [comparison](../solver-comparison.md).


## Acceptance example

These are target examples. Existing tests cover some cases only in part; see [implementation status](../implementation-status.md). Other findings may coexist. See [rule sources](../rule-sources.md) for source editions. Independent human domain review is required before claiming complete correctness.

- **AC-06 — Optimizer and uncertainty:** Within an explicitly complete Masters candidate domain obeying the Axel requirement, excluding all Axel types yields proven infeasibility only if the search proves it. On a small independently enumerated domain, returned optima match all distinct maxima when enumeration is proven complete. A time limit with a feasible incumbent reports unproven optimality; without one it reports incomplete, not infeasible. Unknown rules or unsupported input cannot yield a full compliance or exact-score claim.

## Exhaustive Bronze reference domain


`enumerate_bronze_reference` checks a bounded domain of one to three clean solo 1Lz elements. It uses `Program.elements` in order as candidate identity and collapses empty internal positions. Excluding 1Lz makes the domain infeasible. The objective is the sum of **unfactored** base values; no skating times or second-half credit are assumed.

The ruleset is the 16 July 2026 Adult announcement, Bronze Free Skating p. 11; the table is Communication 2786, 2 June 2026, p. 2, BASE column. The validator rejects a third 1Lz; the lookup gives 0.60 per jump. The maximum is 1.20 and the distinct optimal sequence `[1Lz, 1Lz]` appears once. Clean recognition is assumed. This is a bounded planning result, not an exact score for a performance.

The result covers only implemented Bronze jump checks and the stated domain. Spins, sequences, transitions, execution, recognition uncertainty, timing exceptions, other rules and complete program eligibility remain unverified or out of scope. This is neither full ISU compliance nor an official or global maximum. Unknown validation or base values abort the search; they do not establish infeasibility or an optimum. Source links and open applicability questions are in [rule sources](../rule-sources.md).


## Bounded Pumpkin optimizer


`optimize_bronze` accepts a maximum of up to three elements, the supported ruleset and table identities, an optional 1Lz exclusion, an optional solver runtime limit and an equal-optimum result limit. The non-empty requirement belongs to this domain. It models no spin, step, other jump or recognition. Results record the assumptions, objective, sources and partial validator coverage. Distinctness is based on the ordered elements, not internal solver assignments.

The solver maximizes the number of 1Lz elements, equivalent to maximizing their positive table BV. Its finite feasibility relation is generated from the validator, and each returned candidate is validated and scored again. An optimization proof and complete enumeration are separate: enumeration is complete only when Pumpkin exhausts the fixed-optimum model. A result cap or timeout leaves enumeration unproven. A stopped optimization with an incumbent is feasible but unproven; interruption without an incumbent is incomplete. The time budget starts after model construction and is shared by optimization and enumeration. A zero result limit or unrepresentable deadline is rejected. This slice does not establish complete Bronze compliance, an official maximum, or an exact time-dependent score. Pumpkin 0.5 requires a newer Rust toolchain; WebAssembly compatibility and external proof certificates remain unverified.

The optimizer result records the requested time and solution limits and whether equal-optimum enumeration was requested. Its termination reason distinguishes an infeasibility proof, interrupted optimization, disabled enumeration, a reached solution cap, interrupted enumeration, and exhausted enumeration. An interruption reason does not assert that the time limit caused it. The time budget excludes model construction.

## Category-aware jump optimizer

`optimize_jumps` uses one request for Bronze, Silver, Gold and Masters. The caller explicitly lists clean single or double jumps; each supplies a solo element. The caller also lists the desired clean, two-jump combinations. Only combinations with a verified direct second jump (1T, 2T, 1Lo or 2Lo) and a supported catalog value are accepted. Three-jump combinations, Euler, sequences, other transitions, marks and unlisted variants are not generated. Unsupported requested variants produce an error. Individual exclusions can remove an exact jump or every rotation of a jump kind, from solos and compounds alike.

The request supplies a maximum of up to five elements, a separate optional nonempty restriction, a maximum of 100,000 validator-checked solver proposals, a solution limit and an optional shared time limit. The category-specific count and other implemented jump restrictions come from the shared validator; a combination occupies one element while its components count individually for repetition. Unknown validation preserves the sequence and source-linked findings; an unsupported table value aborts the request. Identical ordered element sequences are equivalent. Empty subsections are considered unless the caller requires a nonempty one; Masters still requires an Axel type.

Pumpkin models each ordered element slot and its table value directly. Empty slots form a suffix. If the shared validator rejects a proposed sequence, the solver excludes that assignment and continues; no separate category-rule model is maintained. The proposal limit covers optimization and tie enumeration; the time budget starts before universe construction and is shared with both solver stages. The result records the request, actual element universe, omitted variants, implemented rule IDs, sources and number of proposals examined. It distinguishes search interruption and a feasible unproven incumbent from an optimum or infeasibility proof. Complete enumeration of ties is reported separately and requires the solver to exhaust the fixed-optimum model; reaching the result limit does not prove completeness. Each returned candidate is checked against every request restriction, validated and scored again. Small independent fixtures enumerate feasibility and ties for all four categories.

All results assume clean recognition and cover only the finite modeled jump subsection. They are not a complete Free Skating optimum or proof of official compliance. GOE, TES, PCS, timing, second-half credit, physical ability, execution and unsupported scoring consequences are outside this objective. Source and fixture review remains necessary.
