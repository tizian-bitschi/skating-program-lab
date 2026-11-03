# Locked program completion (#66): API proposal

Status: public interface approved on 2026-11-03. Additive, optimizer-feature
gated API; existing optimization and assessment requests remain unchanged.

## Entry point and request

`complete_program(registry, request)` accepts `ProgramCompletionRequest` and
returns `Result<ProgramCompletionResult, ProgramCompletionError>`.

The request carries exact `Season`, `ProfileId`, `ScoringTableId`, `ProgramPolicy`,
an ordered `original: Vec<CompletionEntry>`, finite `variants: Vec<ProgramVariant>`,
personal exclusions, insertion policy, minimum/maximum additions, and the existing
objective, proposal budget, solution cap, tie-enumeration flag and time limit.
Generated candidates from #65 enter through their existing variant list; their
generation result remains attached as optional `generated_domain` provenance.
Its variants and profile/SOV identity must match the request. Generator
completeness stays separate from proof over the supplied finite completion domain.

`CompletionEntry` contains a unique caller-supplied string ID, a
`ManualProgramEntry`, and an explicit `locked` flag. Every original entry is
retained in its original relative order; locked entries additionally forbid
replacement of their plan, evidence and recognition assumptions. This initial
completion operation never replaces unlocked entries either. Editing or deleting
an unlocked entry remains a caller action before submitting the draft.

`CompletionInsertionPolicy` declares allowed original boundaries `0..=n`:
boundary 0 is before the first entry, n is after the last, and i is between
entries i-1 and i. Multiple additions may occupy a boundary. A separate optional
absolute-position map fixes selected anchor IDs to zero-based completed indices;
it does not redefine relative-order locking. Invalid boundaries, duplicate IDs
and inconsistent absolute positions are request errors.

Exclusions apply to additions by default. A distinct global-exclusion list also
checks originals and reports conflicting anchor IDs. Exact-variant exclusions
refer only to the addition domain; global restrictions use element predicates.
Combinations and sequences stay atomic. Index-bearing recognition references are
checked against the original index and rebound to the completed index in output;
all other supplied evidence and assumptions remain unchanged.

## Result and failures

`ProgramCompletionResult` retains the request, original draft assessment, domain
audit, source/rule trace, optimizer status, termination reason, proposal count,
best-BV proof and independent enumeration-completeness flag.

Each `CompletedProgramCandidate` includes the full ordered program and evidence,
per-element BV, an assessment using the #64 entry point, and ordered provenance:
`Original { id, original_index, locked }` or
`Inserted { variant_index, boundary, ordinal }`. It reports existing, added and
total unfactored BV separately. Unresolved original BV stays explicitly absent;
it is never replaced by zero. GOE scenarios remain assessment data and do not
enter the objective.

Typed failures distinguish malformed requests, global-exclusion conflicts,
confirmed immutable violations, and unknown/unsupported original evidence.
Missing mandatory elements in a draft remain completion targets. Infeasibility
is proven only after finite-domain exhaustion with fully supported fixed data.
Timeouts and caps preserve valid incumbents and leave tie completeness false.
Solution identity uses the visible ordered plan and relevant evidence, not
boundary labels, duplicate anchor assignments or empty solver slots.

## Implementation and verification

Extend the existing ordered Pumpkin model with original-entry placement and
boundary constraints. Reuse candidate preparation, exclusions, shared whole-plan
validation and catalog scoring. Validate the full completed order for every
proposal, including once-per-program feature consumption. Reassess every returned
completion through `assess_manual_program`; numerical totals must agree.

Independent tiny fixtures enumerate Silver words around fixed 1A and SSp1 and
compare the complete optimal sequence/BV set. Further fixtures cover feature
ordering, compound repetition across anchors/additions, family caps, missing
mandatory elements, illegal and unknown anchors, global conflicts, absolute
positions, Gold/Masters requirements, unsupported domains, duplicate visible
solutions, interruption and result/proposal caps. Run formatting, Clippy, default
and no-default-feature tests before PR creation. Update the optimizer contract,
implementation status and architecture decision record with the approved scope.
