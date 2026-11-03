# Candidate generation API proposal — issue #65

Status: additive public interface explicitly approved by the user on 2026-11-03.

## Additive public contract

Add `generate_program_candidates(registry, request)` behind the existing
`optimizer` feature. Keep all current public signatures and struct fields intact.
The result supplies `Vec<ProgramVariant>` directly usable as the existing
`ProgramOptimizationRequest.variants`.

`CandidateGenerationRequest` carries exact season, ProfileId, ScoringTableId,
discipline and segment, selected generator families, allowed jump kinds and
rotations, typed personal exclusions, spin templates with explicit positions,
revolutions and qualified feature evidence, step/choreographic templates,
optional personal limits, manual variants, maximum candidates and a deterministic
expansion budget. Empty allow-lists mean an explicitly empty selection.

`CandidateGenerationResult` contains the stable ordered deduplicated variants
and `CandidateGenerationAudit`: exact identities and sources, declared supported
domain, generated families, exclusion/omission reasons, recognition and feature
qualification assumptions, applied limits and domain-completeness status.
Completeness applies only to the explicitly declared finite supported domain.
A candidate cap or expansion cap marks that domain incomplete. Unsupported
mechanisms/profiles return an error, distinct from a supported empty domain.

`CandidateGenerationError` distinguishes invalid/contradictory requests,
unsupported mechanisms/profile selection and invalid manual evidence.

## Behavior and module boundaries

Use one new candidate-generation module. Reuse the existing profile policies,
personal-exclusion predicate, element preparation/score checks and shared
validator. Internal visibility changes may expose those helpers within the crate;
they do not add another validator or optimizer.

Enumerate clean solos and a bounded source-verified direct-combination/Axel-
sequence grammar. Never generate Euler or unspecified transitions. Apply filters
to every component before admitting a compound. Keep program-wide repetitions,
feature reuse, family counts and Masters Axel requirements in the shared complete-
program validator.

Generate regular-spin base plans from supported explicit position/revolution
contracts. Higher levels require supplied supported qualification evidence;
unsupported variation prerequisites receive omission reasons. Step/choreographic
templates follow the same explicit evidence rule. Catalog presence alone grants
no eligibility or level. Manual variants use the same checks and filters.

## Validation and documentation

Independent tiny-domain oracle fixtures check exact candidates and stable order,
including equal-value alternatives. Cover all four Adult profiles, component
filters, flying/Layback/feature exclusions, missing qualification, unsupported
profiles, empty domains, caps, semantic deduplication and direct optimizer use.
Run cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test and
cargo test --no-default-features. Update bounded-program-optimizer.md,
implementation-status.md and the current coverage matrix.
