# Architecture

[Assumed recognition](specs/assumed-recognition.md) resolves exact SOV selections before invoking the shared program validator. The evaluation scope keeps structural checks common and runs technical evidence qualification only for the detailed path.

## Current state

The repository contains a Rust library, a separate solver-comparison executable,
and a same-origin C2 browser application with a native HTTP service. The library
accepts caller-provided plans, observations and panel recognition. It returns
validation findings, scoring scenarios, bounded legacy optimization results and
incremental complete planning-search sessions. C2 adds repertoire controls,
ordered manual assessment, exact connection exclusions, one browser-local saved
workspace, and [native proved program adoption](c2-search.md). Live search state
is retained in service memory; it is not a durable checkpoint. There is no rule
database or automatic panel inference. [Implementation status](implementation-status.md)
defines current support.

The core is intended for native and eventual WebAssembly use. Correctness and traceability take priority, followed by independent tests, reproducibility, maintainability and measured performance. WebAssembly compatibility is not established.

## Modules and data flow

| Path | Responsibility and interface |
| --- | --- |
| `src/lib.rs` | Public re-exports, ruleset and table identities, source references. |
| `src/planning_search.rs`, `src/planning_search/model.rs` | Complete theoretical registered-policy repertoires, exact required occurrences, semantic exclusions and atomic-count Pumpkin proof; an owned session retains the solver for incremental collection and shared manual-BV reconstruction. |
| `src/plan.rs` | Ordered `Program` and intended elements; incomplete plans are representable. |
| `src/personal_limits.rs` | Shared typed repertoire maxima and indexed personal conflicts, independent of solver, scoring and competition policies. |
| `src/assessment.rs` | Additive manual-program entry point; combines existing validation and scoring while preserving intended and supplied-recognition stages, optional GOE and incomplete aggregates. |
| `src/legality.rs` | Additive stage-aware coverage report built from shared plan validation, explicit profile policies and unevaluated-rule inventory. |
| `src/execution.rs` | Observations and supplied panel recognition, calls and notation. |
| `src/spin_validation.rs` | Additive indexed spin evidence, explicit source-bound spin policies and partial family coverage. |
| `src/validation.rs` | `validate_program` returns findings and partial coverage for a plan. |
| `src/catalog.rs`, `src/catalog/` | Immutable versioned SOV data and exact-code lookup; source-backed presence does not establish engine support. |
| `src/scoring.rs`, `src/scoring/execution_evidence.rs`, `src/scoring/panel.rs` | Shared catalog resolution, hypothetical judge scores and separate submitted-panel replay; TES and deductions stay separate. |
| `src/optimizer.rs` | Exhaustive reference using validation and base-value lookup. |
| `src/optimizer/pumpkin.rs` | Feature-gated bounded solver using the reference domain, validation and scoring. |
| `solver-comparison/` | Separate executable comparing solvers against the reference, not a library dependency. |

Plan, execution, recognition and scoring are distinct stages. Optional typed execution evidence is supplied separately to additive scoring entry points; the validator checks targets and supported consequences without inventing or mutating panel calls. Missing facts and unsupported cases remain explicit. The [scoring](specs/scoring.md) and [optimization](specs/optimization.md) contracts define outputs and limits.

## Dependency rules

Consumers depend on the core. Validation and scoring use domain types; optimization may use both. Domain logic must not depend on UI, network, storage or solver infrastructure. The `optimizer` feature enables Pumpkin by default; clients can disable default features. Check solver constraints that repeat domain semantics against the validator and independently enumerated small cases.

Version rules and tables without mutating earlier versions. Use integer or exact decimal arithmetic and source-defined rounding. Keep interpretations source-linked and return uncertainty when an applicable rule is unresolved. Do not add a general rule engine, DSL, plugin system or microservices without a new decision.

## Remaining architecture work

The C2 service is an existing consumer of the library. Broader execution
validation and scoring remain future work. The [Pumpkin ADR](adr/0001-pumpkin.md)
records the bounded solver choice. Domain and architecture changes require human
review; passing tests alone does not establish domain correctness. The
[source review](rule-review.md) records existing review waivers.

## Versioned rule profiles

The approved [Senior contract](specs/issue-77-senior-profile-proposal.md) adds
source-bound Women/Men Free Skating identities. `src/senior.rs` owns their
shared technical definition and clean second-solo context. Named
`SeniorJumpPlanV1` bindings extend the shared validator; `SeniorScoringV1`
selects program-context credit. Public struct layouts remain unchanged.
Generation and mixed/locked optimization use the same Senior program policy;
`profiles.rs` resolves built-in program/spin policies and rejects overrides at
both public validator boundaries. Senior search and locked completion allow
eleven slots; other existing finite search limits remain unchanged.
Catalog values and the unfactored optimizer objective remain separate from
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


## Session acceptance boundary (#143)

The public `PlanningSearchSession` and fresh `assess_manual_bv_with_step_limit`
are the acceptance boundaries. Tests compare source-derived maximum literals and
independent finite composition sets, including registered policies. They inspect
required provenance and exact whole-element identities across batches and pause,
without depending on solver assignments or delivery order. The session owns its
solver and immutable request, borrows the selected registry and releases its state
when dropped. Legacy APIs remain capped at five deliveries. See the
[contract](specs/planning-search.md#incremental-search-sessions-142) and
[acceptance evidence](planning-session-acceptance.md).

## C2 application adapter (#155)

`service/` is a standalone Axum/Tokio consumer of the library. Its strict
versioned HTTP DTOs translate context and personal settings into core selection.
Ordinary assessment uses `spawn_blocking`; each live search has a dedicated native
owner thread that constructs and retains its non-Send registry/session. Independent
control flags reach busy work without waiting behind its command queue. Safe
handles/DTOs cross the HTTP boundary, and live-owner capacity is released only
after actual owner termination. `web/` contains Vue/TypeScript/Vite
presentation and canonical backend element keys. Built assets and `/api/v1`
share the service origin. Profile registration and eligibility remain in Rust;
TypeScript contains translations and presentation state only. The library
retains its existing dependencies and callers. See [C2 repertoire](c2-repertoire.md).

`App.vue` owns the persisted workspace inputs and local-only fader previews.
`ManualEditor.vue` owns stable ordered occurrences and revision-correlated
automatic assessment, with a native modal element/connection picker.
`PlanningSearch.vue` owns page-held search capabilities, cursor reconciliation,
controls and recovery. Locale/layout changes do not alter its problem signature.
`ResultList.vue` pages both confirmed programs and selected comparison previews in
groups of twenty. This bounds rendered rows, not collected data or the total
number of alternatives: all received server IDs/data remain available during the
page session. Append does not replace the current page or move focus. The small
localStorage workspace excludes search capabilities and result history. See
[C2 acceptance observations](c2-acceptance.md) for measured growth and remaining gates.

