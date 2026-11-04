# Architecture

## Current state

The repository contains a Rust library and a separate solver-comparison executable. It accepts caller-provided plans, observations and panel recognition. It returns partial validation findings, scoring scenarios and bounded optimization results. It has no UI, network service, persistence, rule database or automatic panel inference. [Implementation status](implementation-status.md) defines current support.

The core is intended for native and eventual WebAssembly use. Correctness and traceability take priority, followed by independent tests, reproducibility, maintainability and measured performance. WebAssembly compatibility is not established.

## Modules and data flow

| Path | Responsibility and interface |
| --- | --- |
| `src/lib.rs` | Public re-exports, ruleset and table identities, source references. |
| `src/plan.rs` | Ordered `Program` and intended elements; incomplete plans are representable. |
| `src/assessment.rs` | Additive manual-program entry point; combines existing validation and scoring while preserving intended and supplied-recognition stages, optional GOE and incomplete aggregates. |
| `src/legality.rs` | Additive stage-aware coverage report built from shared plan validation, explicit profile policies and unevaluated-rule inventory. |
| `src/execution.rs` | Observations and supplied panel recognition, calls and notation. |
| `src/spin_validation.rs` | Additive indexed spin evidence, explicit source-bound spin policies and partial family coverage. |
| `src/validation.rs` | `validate_program` returns findings and partial coverage for a plan. |
| `src/catalog.rs`, `src/catalog/` | Immutable versioned SOV data and exact-code lookup; source-backed presence does not establish engine support. |
| `src/scoring.rs`, `src/scoring/execution_evidence.rs` | Base values and scores from recognition and supplied facts; TES and deductions stay separate. |
| `src/optimizer.rs` | Exhaustive reference using validation and base-value lookup. |
| `src/optimizer/pumpkin.rs` | Feature-gated bounded solver using the reference domain, validation and scoring. |
| `solver-comparison/` | Separate executable comparing solvers against the reference, not a library dependency. |

Plan, execution, recognition and scoring are distinct stages. Optional typed execution evidence is supplied separately to additive scoring entry points; the validator checks targets and supported consequences without inventing or mutating panel calls. Missing facts and unsupported cases remain explicit. The [scoring](specs/scoring.md) and [optimization](specs/optimization.md) contracts define outputs and limits.

## Dependency rules

Consumers depend on the core. Validation and scoring use domain types; optimization may use both. Domain logic must not depend on UI, network, storage or solver infrastructure. The `optimizer` feature enables Pumpkin by default; clients can disable default features. Check solver constraints that repeat domain semantics against the validator and independently enumerated small cases.

Version rules and tables without mutating earlier versions. Use integer or exact decimal arithmetic and source-defined rounding. Keep interpretations source-linked and return uncertainty when an applicable rule is unresolved. Do not add a general rule engine, DSL, plugin system or microservices without a new decision.

## Planned architecture

A UI, native host or service may consume the library later. Full validation, complete scoring and broader optimization domains are planned. The [Pumpkin ADR](adr/0001-pumpkin.md) records the bounded solver choice. Domain and architecture changes require human review; passing tests alone does not establish domain correctness. The [source review](rule-review.md) records existing review waivers.

## Versioned rule profiles

The approved [Senior contract](specs/issue-77-senior-profile-proposal.md) adds
source-bound Women/Men Free Skating identities. `src/senior.rs` owns their
shared technical definition and clean second-solo context. Named
`SeniorJumpPlanV1` bindings extend the shared validator; `SeniorScoringV1`
selects program-context credit. Public struct layouts remain unchanged.
Generation and mixed/locked optimization use the same Senior program policy;
catalog values and the unfactored optimizer objective remain separate from
contextual credit. See the [source review and coverage matrix](rules/isu-senior-2026-27.md).

The accepted [profile ADR](adr/0002-rule-profiles-proposal.md) defines the additive
migration. `src/profiles.rs` owns profile identities, immutable registration,
source bindings and explicit event overrides. Validation, numerical scoring and
the bounded jump solver share profile policy through additive entry points;
legacy APIs select built-in Adult definitions. SOV values remain in the catalog.
Unknown identities retain the request and error reason. Missing sources,
conflicting overrides and unsupported mechanisms never fall back to Adult rules.

The additive `program_validation` module checks bounded ordered mixed-program evidence using shared jump/spin validation. `optimizer/programs.rs` consumes that validator and catalog scoring; the Pumpkin ordered-slot/value helper is shared with the jump optimizer. See [mixed-program contract](specs/bounded-program-optimizer.md).

`optimizer/completion.rs` preserves caller-identified manual originals and maps
them into the same ordered-slot model with anchored positions and insertion-gap
constraints. It reconstructs provenance and reuses manual assessment for returned
completions; see [ADR 0003](adr/0003-immutable-program-completion.md).

The versioned `AdultDefinition` in `profiles.rs` is the single built-in policy-data contract for jump, spin and mixed-program adapters. Family sources are explicit. Validators record source-bound `AppliedRule` entries while checking evidence; the optimizer distinguishes this trace from its available-rule inventory.
