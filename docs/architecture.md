# Architecture

## Current state

One Rust library models season, Adult singles categories, supported ruleset and scoring-table identities, source references, and ordered planned elements. Source applicability and consistency between fields are not validated. Jump transitions are distinct from listed jumps; regular spins distinguish flying and non-flying entries and retain position and foot-change order. Choreographic spins are separate elements. Optional spin details distinguish unspecified from explicitly empty lists. Feature attempts are descriptions attached to positions, not recognized features; other phases are not yet modeled for features. Spin revolutions currently record whole counts only. The model permits incomplete or illegal plans and does not validate them. There is no rule database, validator, scoring, solver, API or UI. Independent domain review is required. CI checks formatting, Clippy and tests. The intended scope and source limitations are in [requirements](requirements.md) and [rule sources](rule-sources.md).

## Intent and boundaries

The application accepts program and execution data, and returns validation findings, score breakdowns or optimization results. It does not act as a Technical Panel or judge. A future UI, native host or service is outside the domain core. The core should be usable natively and in WebAssembly, subject to checking dependencies for the target.

Priorities: (1) correctness against applicable sources, (2) traceability and explicit uncertainty, (3) independent testability and reproducibility, (4) maintainability and portability, (5) performance when measured.

## Planned responsibilities and flow

Start with modules within one crate, introduced only when implemented:

- **Domain and rules:** explicit representations for ordered plans, observed execution, supplied panel recognition and source-linked ruleset and table identities. These stages remain distinct.
- **Validator:** local and program-wide checks, returning separate plan findings, recognition/input inconsistencies and unknowns with source locations and affected inputs. It does not infer every panel decision or spin level.
- **Scoring:** deterministic scenario calculation from supplied recognized code, calls and a final single-judge GOE, using versioned table values. Return element breakdowns and TES separately from deductions; PCS and panel aggregation are future work.
- **Optimizer:** search a finite request-defined candidate space under official and personal constraints. Report proof and enumeration status, including incomplete or infeasible outcomes. Validate candidates and recompute their base values through the core.

Input and selected version flow into domain structures; validation reports findings and unknowns. Supplied recognition, final hypothetical GOE and scoring tables feed scoring, while fall and timing facts feed separate deductions. Optimization consumes the same domain and rule semantics, then submits candidates to validation and objective calculation. A proof is limited to the modeled domain and constraints. Consumers depend on the core; validator and scoring depend on domain/rules, optimizer may use all three. Domain logic must not depend on UI, network, storage or solver-specific infrastructure. Avoid duplicated rule interpretations; if solver constraints duplicate semantics, test them against the validator and independently enumerated small cases.

## Decisions and quality assurance

Rust and explicit data types support testable domain logic and native/Wasm reuse. Use fixed-point or integer arithmetic with documented rounding. Version rules and tables without mutating prior versions. Do not build a general rule engine, DSL, plugin system or microservices. Compare Pumpkin, Z3 and, if appropriate, OR-Tools on the same small concrete problem before choosing a solver or infrastructure; include optimality, alternative enumeration, infeasibility, deployment and Wasm constraints in the comparison.

Work proceeds: requirement → acceptance criteria → small implementation → tests → review → integration. Derive acceptance fixtures independently from official documents; passing tests alone does not establish domain correctness. Keep each change buildable; CI is mandatory. Domain and architecture changes also require human review. Run formatting, Clippy and relevant tests locally, and document relevant decisions and limitations with the change.
