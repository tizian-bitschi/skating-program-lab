# Requirements

## Purpose and scope

Skaters and coaches need to plan and compare technical programs; developers need reproducible rules and results. The MVP targets singles Free Skating at Bronze, Silver, Gold and Masters under the 2026/27 ISU-supported International Adult competitions announcement, within its stated events and dates. It is a planning aid, not an official panel or judging system. See [rule sources](rule-sources.md) for the authority and unresolved questions.

The MVP excludes artistic skating, pairs, dance, national variations, other seasons and categories, component scoring, accounts, persistence, a user interface and an HTTP API. Additional seasons, categories and national rules are future work; they must not overwrite earlier versions. Expected TES optimization and individual judges' panel simulation are later extensions.

## MVP behavior

- Select an explicit ruleset and scoring-table version. Represent an ordered program and distinguish the planned element, observed execution, and panel-recognized element and calls; do not silently equate them.
- Validate local element rules and global program constraints, including jump combinations and sequences, jump repetitions, spin positions, features and dependencies. Report all detected violations with stable identifiers, affected elements, a plain explanation and the specific official source location. Report incomplete or unsupported input as unknown/error, never as compliant.
- Simulate a supplied recognized performance using the appropriate base values, levels, calls, GOE and rounding. Distinguish invalid input, non-compliance, element value reductions or zero values, and program deductions. Show element base value, adjustments, GOE and final element score, summed TES, and applicable deductions separately; do not subtract deductions from TES or claim a total segment score without PCS. Do not infer judges' GOE from a fall automatically unless the applicable rule explicitly requires it.
- Optimize total base value over a finite, explicitly specified candidate domain and its assumptions. Support exclusions of jumps, spin positions and features, multiple equal optima, infeasible constraints and an explicit incomplete/timeout status. Never claim global optimality or completeness without proof. Revalidate each returned candidate with the same ruleset and independently recompute its objective.

## Acceptance criteria

These describe future behavior, not current functionality. Expected fixtures must be independently derived from the cited rules and reviewed by a domain expert.

1. **Validator — local and global:** Under the Adult 2026/27 Bronze rules, a 1A is reported as prohibited; five jump elements trigger the maximum-four violation, while four do not trigger that particular violation. Each finding identifies its element(s), rule and announcement p. 11. A program containing both conditions reports both. This does not imply that any four-jump program is otherwise valid.
2. **Validator — structure:** Source-derived boundary fixtures cover combination length, repeated listed jumps, spin positions and dependent features for each category. For every rule, valid and invalid neighbors produce the expected finding; unsupported facts yield an explicit unknown result. A violation does not automatically imply a zero score.
3. **Scoring:** For reviewed fixtures spanning a clean element and applicable `q`, `<`, `<<`, `!`, `e`, level and zero-value cases, the output matches source-derived element values and prescribed rounding step by step. In Bronze, a fall inside an element has no separate fall deduction, while a fall outside an element has a 0.5 deduction (announcement p. 11); TES and deductions remain separate. A fall can affect the element GOE even without a separate deduction: Communication 2788 p. 14 specifies a -5 GOE reduction and p. 16 limits the starting GOE. Verify the complete calculation rather than equating that reduction to a final GOE. Reject incompatible or unsupported calls with an explanation instead of guessing.
4. **Optimization:** On a small, fully enumerated ruleset and candidate set, match the independently enumerated maximum and return all distinct equal optima when requested and completion is proven. Every result passes validation and respects each exclusion. Contradictory constraints report infeasibility only when proven; interrupted search reports incomplete, not infeasible or optimal. State the equivalence definition, candidate scope and any enumeration limit in the result.
5. **Versioning:** Identical input and explicit versions yield identical results. An unknown version is rejected; adding a season leaves previously versioned fixtures unchanged and traceable to their original sources.

## Quality requirements

Correctness and source traceability take priority over convenience and speed. Computation must be deterministic and use exact decimal or integer arithmetic with source-defined rounding. Rules and tables are versioned and source-linked; uncertainties block claims of compliance or exact scoring. Pure domain functions permit independent tests and eventual native and WebAssembly use. Keep modules small, dependencies justified and changes reviewable. No performance target is asserted before a representative workload exists.
