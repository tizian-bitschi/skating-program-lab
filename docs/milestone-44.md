# Versioned catalog and reusable Single Skating foundation (#44)

This audit covers `main` through PR #61. Issues #45–#49 are closed and their
implementation PRs are merged. The milestone delivers reusable, bounded
planning mechanisms. It does not establish complete Single Skating rule coverage.

## Acceptance and coverage matrix

| Area | Delivered support and evidence | Boundary |
| --- | --- | --- |
| Catalog data (#45, PRs #50–#52) | One immutable June 2, 2026 Communication 2786 catalog: 625 exact codes across 14 families, including 223 Pair-specific rows. `tests/catalog_manifest.rs` checks a separate source inventory; `tests/catalog.rs` checks all 6,875 printed numerical cells and 42 hand-read all-column fixtures. | Catalog presence is data availability, not eligibility or implemented scoring. Notes and PCS factors remain data. See the [catalog audit](catalog.md) for the pinned hash, graphic-label exceptions and review limits. |
| Single scoring (#46, PRs #53–#55, #57) | Shared catalog lookup for single/double jumps and supported calls, supplied combinations/sequences/Euler, and all 187 representable regular/V spin, step and choreographic rows. `tests/scoring.rs` and `tests/execution_scoring.rs` preserve separate deductions, partial totals and zero/no-value/ignored distinctions. | Supplied recognition and final hypothetical single-judge GOE are inputs. Triple/quad/quint and Pair scoring, PCS and panel averaging are unsupported. Some execution consequences remain unresolved; no missing value becomes an exact zero. |
| Rule profiles (#47, PR #59) | Exact versioned profile identity, sources, scoped overrides and typed policy feed shared consumers. Four Adult profiles retain compatibility adapters. Synthetic fixtures in `tests/profiles.rs` and `tests/program_optimizer.rs` add a category by registration, without extending `AdultCategory` or creating another scorer/solver. | Synthetic rules are caller assertions. No official Junior, Senior or national category is registered. Unknown identities, unsupported mechanisms and conflicting bindings are rejected. |
| Plan validation (#48, PR #60 and prerequisite work in #61) | Shared jump and regular-spin checks plus ordered feature reuse, first-attempt/feature-10-first-success rules, per-foot/level credit, Gold step restrictions and distinct ChSq/ChSp evidence. `tests/validation.rs`, `tests/spin_validation.rs` and `tests/program_validation.rs` cover supported neighboring valid/invalid/unknown cases and source-bound traces. | Typed evidence and local qualification assumptions do not infer execution or panel calls. Difficult-variation categories, some feature prerequisites, Euler consequences and unmodeled transitions remain Unknown/Unsupported. The older spin-only entry points retain their narrower scope. |
| Bounded optimization (#49, PR #61) | One mixed-program solver consumes shared validation, profiles and catalog scoring. Independent finite-word oracles in `tests/program_optimizer.rs` verify objectives and ordered ties for all four Adult profiles, synthetic policy, competing features and personal exclusions. | Proofs cover only the explicit finite conditional model. Missing recognition/qualification or unsupported requested variants are rejected. Search interruption, infeasibility and incomplete tie enumeration stay distinct. No official/global optimum or physical skateability is established. |

Successful checks remain partial. `validate_supported_program` checks supported
mixed evidence; `validate_program` remains the legacy jump-plan interface.
`validate_program_with_evidence` adds individual regular-spin checks, without
the full mixed-program feature accounting. Callers must choose the interface
whose documented evidence and coverage match their task.

## Source editions and precedence

The catalog is Communication 2786 **updated June 2, 2026**, not its earlier May
edition. Existing Adult profiles use the **July 16, 2026** Adult announcement
for the named 2026/27 events. Supplementary implemented semantics cite
Communication 2788 **May 19, 2026**, Sports Rules **2026**, the Single Technical
Panel Handbook **July 15, 2026**, the Referees/Judges Handbook **2026/27**, and
Q&As 1–3 **April, May and October 2026**. Exact URLs and page locations remain
in [rule sources](rule-sources.md) and returned source references.

The Adult announcement's precedence is qualified by special Adult comments in
Communications. These sources do not authorize extrapolation to other categories.
Source-backed catalog values, eligibility policy, execution evidence, supplied
recognition and numerical scoring keep separate responsibilities.

## Compatibility and extension

Public interfaces were added under the approved [profile migration
contract](adr/0002-rule-profiles-proposal.md), [spin evidence
contract](specs/issue-48-spin-evidence-proposal.md) and [mixed-program
contract](specs/bounded-program-optimizer-proposal.md). Existing plan structs,
Adult entry points, twelve-jump fixtures and stable source references remain.
This audit adds no public API or architecture migration.

To add a category using implemented mechanisms, register a new immutable
`RuleProfile` with its own identity, typed policies, exact SOV edition, citations
and explicit precedence. Bind a matching `RegularSpinPolicy`/`ProgramPolicy` when
using additive evidence or mixed optimization. The synthetic tests demonstrate
this path; copying Adult sources does not establish an official category.
Unsupported mechanisms need separately scoped design and source review.

## Remaining work and review limits

- [#56](https://github.com/tizian-bitschi/skating-program-lab/issues/56):
  touch-down/weight-transfer execution and GOE classification.
- [#58](https://github.com/tizian-bitschi/skating-program-lab/issues/58):
  multiple full revolutions between jumps and their numerical consequences.
- Difficult-variation categories, unmodeled feature prerequisites, unresolved
  Adult Euler/panel consequences and automatic recognition remain outside
  supported evaluation. Unknown evidence does not prove compliance.
- Full Pair rules/scoring/optimization and Ice Dance remain outside this
  milestone. Pair catalog data does not imply any of those capabilities.
- Catalog fixtures use separate extraction/transcription and external AI
  review. Independent human cell-by-cell or full domain approval is not claimed;
  see [source review](rule-review.md). Native tiny-domain solver measurements
  do not establish broad performance or WebAssembly support.

Closing the foundation milestone does not resolve these limits. The detailed
[scoring](specs/scoring.md), [regular-spin](specs/regular-spin-validation.md) and
[mixed-program](specs/bounded-program-optimizer.md) contracts define the supported
subsets. Existing tests provide regression evidence; they do not establish every
technical-panel rule.

## Integration review corrections

The #44 review found two validator coupling points and a combined-document
citation. An internal `FeatureCreditEvaluation` contract now selects deferred
legacy credit or mixed-program evaluation before spin checks run; the mixed
validator no longer removes findings/traces by string ID. Local feature-target
checks run in both modes. The shared jump validator alone evaluates direct
Axel-step assumptions. Its trace cites Communication 2788 (May 19, 2026), p. 3,
separately from the selected Adult category's July 16 announcement binding.
Legacy jump findings and their source references remain compatible; the mixed
entry point no longer adds a duplicate Unknown for the same missing assumption.
`tests/program_validation.rs` verifies both spin scopes, invalid local targets,
one sequence diagnostic and exact independent document/version/page references.
