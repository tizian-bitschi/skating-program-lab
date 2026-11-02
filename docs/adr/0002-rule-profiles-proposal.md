# Versioned rule profiles: schema and migration contract

Status: accepted by the requesting user on 2026-11-02, including additive public interfaces.
Implementation retains existing APIs.
Issue: https://github.com/tizian-bitschi/skating-program-lab/issues/47
Baseline: a27717a3d54034b4fe07c9512639c4292e48b865 (`origin/main`).

## Problem

`Program`, scoring and `JumpOptimizerRequest` select `AdultCategory` and
`RulesetId` independently. Validation embeds category policy in matches;
scoring repeats category-dependent duration and deduction selection. Adding a
category currently requires editing these consumers. SOV catalog identity is
already separate and must remain separate.

## Proposed public contract

Add a `profiles` module with owned `ProfileId` (stable string plus version),
`RuleProfile`, `ProfileRegistry`, `ProfileSelectionError` and `ProfileProgram`.
`ProfileProgram` contains profile identity, explicit SOV table identity and the
existing ordered `PlannedElement` list. A profile contains discipline, segment,
season, competition/federation scope, category identifier, source editions,
rule bindings and explicit overrides. Category identifiers are data, not a new
closed enum. Pair scope is representable but evaluation returns Unsupported.

Registration validates nonempty identities and citations, duplicate identities,
policy bounds and rule bindings. Resolution returns an error retaining the
requested identity and reason for unknown versions, missing sources, ambiguous
precedence or unsupported mechanisms. It never substitutes an Adult profile.
Synthetic provenance is explicit and cannot be labelled official. A changed
policy or source edition requires a new profile version; duplicate registration
cannot overwrite an existing entry.

Add profile-aware validation, program scoring and jump optimization entry
points. Preserve all existing public structs, fields, enum variants, functions
and signatures. Existing callers use adapters selecting one of four built-in
profiles. Do not add fields to existing public structs: that breaks Rust struct
literals. Add separate profile optimizer request/result types; retain the
selected profile identity in results and errors. Shared internal evaluators
serve both APIs. Existing recognition ruleset identities stay intact; an
explicit compatibility binding checks them before profile scoring. Unsupported
bindings return Unsupported rather than reinterpreting recognition.

The final names and signatures are additive public interface changes and are
part of the approval requested by this proposal. No existing API removal or
deprecation is proposed.

## Typed policies and evaluators

Extract the current jump policy into validated data: permitted jump kinds and
rotation ranges, maximum jump elements, compound counts and lengths, sequence
and Euler permissions, repetition limits and the Axel requirement. Keep stable
rule IDs and per-rule citations as bindings in the profile. Duration and
supported deduction policies are typed separately from element eligibility.

Represent family/variant restrictions and level bounds explicitly. Missing
spin-feature, sequence-recognition or other mechanisms remain unsupported
coverage; a bound does not establish recognition support. Use versioned enums
to select the existing named transition, Euler, repetition, timing and scoring
evaluators. Do not encode arbitrary expressions or turn unresolved semantics
into numeric parameters. Unsupported evaluator versions block the affected
result. Optimizers must reject unsupported eligibility policies before search,
and continue checking proposals with the shared validator.

Catalog lookup continues to use only `ScoringTableId` and element keys. Profiles
link to compatible table editions without copying values. A table mismatch
returns an explicit error. Profile eligibility never changes catalog values.

## Sources and overrides

Store exact document editions and locations on each rule binding. Overrides
identify the replaced binding, its replacement citation, event scope and
explicit precedence justification. Resolve bindings before evaluation. Reject
duplicate replacements, cycles, absent targets and unresolved conflicts; vector
order and document recency are not precedence rules.

For the Adult baseline preserve the qualified precedence in
`docs/rule-sources.md`: the announcement prevails only within its stated scope
and subject to Communications containing special Adult comments. Do not infer
applicability to another event. Synthetic overrides carry test provenance and
never become evidence for an official category.

## Adult compatibility mapping

All four profiles retain Singles, Free Skating, season 2026/27, the July 16
Adult announcement and the existing shared general-rule citations. IDs use
`isu-adult-2026-27-<category>` with version `2026-07-16`.

| Category | Jump cap | Compound length cap | Existing eligibility | Category citation |
| --- | --- | --- | --- | --- |
| Bronze | 4 | 2 | Singles without Axel | Announcement p. 11 |
| Silver | 4 | 3 | Singles including Axel | Announcement p. 10 |
| Gold | 5 | 3 | Singles and double T/S/Lo | Announcement p. 9 |
| Masters | 5 | 3 | Singles and doubles; Axel required | Announcement p. 8 |

This table summarizes implemented behavior, not complete category legality.
Preserve compound/Euler/repetition results, unknown transitions, rule IDs,
source locations, partial coverage, duration and current deduction limitations
from existing fixtures. The adapter is the sole `AdultCategory` mapping;
consumer evaluation does not match on category identity.

## Bounded implementation and proof

1. Review this contract before implementing public APIs or restructuring plan
   selection. Keep `src/plan.rs` intact apart from additive types if needed.
2. Introduce registry/selection and the four immutable Adult definitions;
   migrate shared validation policy through compatibility adapters.
3. Add profile-aware scoring and optimizer selection using shared internals.
   Add a synthetic profile by registration alone, using supported jump rules:
   for example, restrict a test category to one jump element. The same two
   clean solo jumps then violate its cap while retaining Adult count behavior.
   Also demonstrate an eligibility difference with an explicitly restricted
   jump-kind policy. Never identify this profile as Junior or national rules.

Tests must independently cover unknown identity retention, duplicate versions,
source citations, deterministic overrides and ambiguity, unsupported evaluator
versions, unchanged Adult outcomes and source IDs, shared catalog values,
legacy struct-literal compilation, profile scoring consistency and optimizer
rejection/selection. Derive expected Adult outcomes from existing source-backed
fixtures, not the extracted policy. Run `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` and
`cargo test --no-default-features` for each implementation slice.

Update architecture, requirements, implementation status and source guidance
with implemented coverage in the corresponding slices. New official profiles
require verified authoritative editions, applicability and precedence review,
source-derived fixtures and independent review before registration as supported.
The external GPT-6 review remains required before merge. This proposal alone
does not complete or close issue #47.

## Implementation details

The public API uses `ProfileJumpOptimizerRequest`, `ProfileJumpOptimizerResult`,
`ProfileValidationResult` and `ProfileProgramScore`. Registry entries are owned,
immutable after registration and inspected through `get`; `resolve` additionally
checks evaluator support. Definitions are Rust policy data; file deserialization
and dynamic rule identifiers are outside this slice. Stable rule identifiers
remain static strings for compatibility with `Finding`.

`FamilyPolicy` selects represented element families/variants and required
presence. Level and feature restrictions remain explicit `UnsupportedPolicy`
data and prevent resolution, since recognition semantics are not implemented.
JumpPlanV1 retains partial coverage even when all implemented checks pass.
Overrides replace exactly one base binding; chains are rejected. Policy values
are part of the versioned definition, with replacement bindings supplying their
sources. Registration validates structure and citations, not the authority of
caller-supplied documents. Official onboarding still requires source review.

The shared Pumpkin model retains its documented five-slot safety bound and
finite candidate subset. It checks proposals with the selected profile policy;
no Adult category selects policy inside that search. A private legacy carrier
retains the old request/result representation internally and is converted to
profile programs at the public boundary. No synthetic result exposes an Adult
category. Numerical scoring uses the explicitly selected AdultScoringV1
mechanism and recognition compatibility binding; it does not validate category
eligibility. Unimplemented mechanisms block profile selection conservatively.
