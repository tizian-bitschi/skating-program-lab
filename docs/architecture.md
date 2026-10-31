# Architecture

## Current state

One Rust library contains only crate documentation. There is no rule model, validator, scoring, solver, API or UI. CI checks formatting, Clippy and tests. The intended scope and source limitations are in [requirements](requirements.md) and [rule sources](rule-sources.md).

## Intent and boundaries

The application accepts program and execution data, and returns validation findings, score breakdowns or optimization results. It does not act as a Technical Panel or judge. A future UI, native host or service is outside the domain core. The core should be usable natively and in WebAssembly, subject to checking dependencies for the target.

Priorities: (1) correctness against applicable sources, (2) traceability and explicit uncertainty, (3) independent testability and reproducibility, (4) maintainability and portability, (5) performance when measured.

## Planned responsibilities and flow

Start with modules within one crate, introduced only when implemented:

- **Domain and rules:** explicit types for program, planned/executed/recognized elements, source-linked ruleset and table identities.
- **Validator:** local and program-wide checks, returning findings with source locations and affected inputs.
- **Scoring:** deterministic calculation from recognized elements, calls and GOE, showing intermediate values and separately reporting deductions.
- **Optimizer:** search a stated finite candidate space under official and user constraints, returning proof status and alternatives. Validate candidates and recompute scores through the core.

Input and selected version flow into domain structures; validation reports findings. Recognized execution and scoring tables feed scoring. Optimization consumes the same domain and rule semantics, then submits candidates to validation and objective calculation. Consumers depend on the core; validator and scoring depend on domain/rules, optimizer may use all three. Domain logic must not depend on UI, network, storage or solver-specific infrastructure. Avoid duplicated rule interpretations; if solver constraints duplicate semantics, test them against the validator and independently enumerated small cases.

## Decisions and quality assurance

Rust and explicit data types support testable domain logic and native/Wasm reuse. Use fixed-point or integer arithmetic with documented rounding. Version rules and tables without mutating prior versions. Do not build a general rule engine, DSL, plugin system or microservices. Compare Pumpkin, Z3 and, if appropriate, OR-Tools on the same small concrete problem before choosing a solver or infrastructure; include optimality, alternative enumeration, infeasibility, deployment and Wasm constraints in the comparison.

Work proceeds: requirement → acceptance criteria → small implementation → tests → review → integration. Derive acceptance fixtures independently from official documents; passing tests alone does not establish domain correctness. Keep each change buildable; CI is mandatory. Domain and architecture changes also require human review. Run formatting, Clippy and relevant tests locally, and document relevant decisions and limitations with the change.
