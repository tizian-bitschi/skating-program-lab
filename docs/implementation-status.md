# Implementation status

[Assumed recognition](specs/assumed-recognition.md) supports explicit SOV selections in manual assessment, legality, finite candidate generation, optimization and locked completion. The coverage contract lists the execution checks outside that mode and the Senior repetition limit of the solver.
The [milestone #89 matrix](milestone-89.md) maps the Adult and Senior workflows
to source-backed fixtures and independent solver oracles.
The optional `preview_goe_scenario` API scores the selected program with
published GOE columns, retaining per-element errors and partial totals.

The [Senior Grand Prix parent audit (#75)](milestone-75.md) connects the six
merged workstreams to their tests and limitations. An eleven-slot conditional
plan preserves Ami NAKAI's source jump structure and family order, completes in
a singleton locked domain and is revalidated/rescored at independent BV 58.30.
Movement evidence, clean recognition and Basic roles remain hypothetical.

Issue #81 adds [source-derived Senior native benchmarks](senior-grand-prix-benchmarks.md)
for both Grand Prix events, Women/Men, three domain scales and both full/completion
APIs. Independent small-domain oracles check all 1440 feasible words and complete
optimal tie sets; medium tests preserve source-linked immutable anchors and
literal BV subtotals. Measurements use a pinned runtime/dependency snapshot,
explicit budgets and conservative proof flags. No solver optimization or latency
guarantee is established. Original protocols lack plan motion evidence; these
domains are conditional abstractions, not official program/feature validation.
High-solo repetition across Senior completion/manual scoring remains an explicit
`InconsistentAssessment` boundary; the large completion domain excludes such additions.

Issue #79 audits the shared eleven-slot Senior model, separates the finite
engineering ceiling from Rule 612, and adds full/partial completion, exact tie,
provenance and independent numeric-bound regressions. Existing search and result
caps remain; eleven-slot capacity carries no latency or global-optimum promise.
See the [engineering audit](specs/bounded-program-optimizer.md#eleven-slot-engineering-audit-79).

Issue #78 adds table-backed Senior triple/quad call scoring and optional explicit
multi-judge aggregation. Protocol replay reproduces all 132 independently
transcribed #76 rows, retaining printed-credit provenance and unresolved inputs.
See the [API, numerical coverage and limits](specs/senior-protocol-scoring.md).

Issue #77 adds source-bound 2026/27 Senior Women/Men Free Skating profiles,
Rule 612 jump constraints, distinct spin roles and shared family evidence
validation. Clean ordered program/manual contexts apply second-solo 80%
credit without changing catalog BV or supplied panel data. Candidate generation,
mixed optimization and locked completion resolve Senior identities. The
[Senior source review and coverage matrix](rules/isu-senior-2026-27.md) records
edition bindings, independent tests and unsupported panel/scoring cases.

The [product milestone #63 audit](milestone-63.md) maps the four backend workflows
and merged workstreams #64–#68 to executable acceptance evidence and remaining
limits. Supported finite conditional optimization and partial rule coverage do
not establish an official result or full regulatory compliance.

## Rule coverage (#67)

`assess_program_legality` adds stage-aware rule coverage without changing existing
public report types. It reuses mixed-program validation and the shared manual
assessment obligation classification. Coverage preserves selected event overrides
and includes unevaluated Adult timing, performance and variation mechanisms.
First-success ambiguity and plain-upright feature prerequisites now block definite
feature credit in every mixed-program consumer. See the
[per-profile source matrix](specs/legality-coverage-proposal.md) and
`tests/legality.rs`. Recognition assumptions and unresolved Euler consequences
remain explicit; no covered pass establishes full ISU legality.

## Locked program completion (#66)

`complete_program` preserves caller-identified original entries and inserts finite
supported variants at explicit original boundaries. It extends the existing
Pumpkin ordered-slot search and shared whole-program validation. Results carry
ordered provenance, evidence, catalog BV subtotals, manual assessments and source
traces, with separate optimum and tie-enumeration proof. Unknown originals,
immutable violations and global exclusion conflicts remain distinct. Tests cover
an independent tiny Silver oracle, ordered feature use, repetitions, mandatory
roles in Gold/Masters, exclusions, positions and search limits. See the
[contract](specs/bounded-program-optimizer.md#immutable-ordered-completion-66).
Proof covers supplied finite conditional domains, never full official compliance.

## Manual program assessment (#64)

`assess_manual_program` accepts one profile-selected ordered program with typed
plan evidence and explicit intended-recognition assumptions. It returns sourced
conditional BV, independent optional supplied-recognition/GOE scenarios and the
unchanged mixed-program validation report. Missing GOE never becomes zero; partial
aggregates identify unresolved indices. Draft completion diagnostics remain
separate from established violations. Coverage stays partial. Legacy functions
are unchanged. See the [contract](specs/manual-program-assessment-proposal.md).

The library provides partial coverage. These tests cover selected cases, not every domain rule. See [rule sources](rule-sources.md) for editions and open questions.

The [foundation milestone audit](milestone-44.md) maps the five workstreams of
issue #44 to merged PRs, regression evidence and remaining limits. Issues being
closed does not establish full rule coverage.

## Domain

`src/lib.rs`, `src/plan.rs` and `src/execution.rs` model version and source identities, ordered plans, observations and supplied panel outcomes. `tests/domain.rs`, `tests/plan.rs` and `tests/execution.rs` cover selected representations and errors.

Jump transitions are distinct from listed jumps. Regular spins distinguish flying and non-flying entries and preserve the order of positions and foot changes; choreographic spins are separate. Feature attempts are descriptions attached to positions, not recognized features. An unspecified feature list differs from an explicitly empty one. Planned jump type and rotations may be missing. Observed movement is free text and revolutions are whole counts. Indices refer to caller-provided lists and are not validated. Source applicability and metadata consistency are not validated. Recognition notation and constructor limitations are in the [scoring specification](specs/scoring.md#recognition-representation).

## Validation

`src/validation.rs` checks jump eligibility, element count, combinations, sequences, Euler and listed-jump repetitions for Bronze, Silver, Gold and Masters. Masters requires an Axel-type jump. Transitions outside the documented direct subset, direct sequence steps, and Euler footwork or panel consequences remain unknown; other seasons are unsupported. Findings carry a rule identifier, explanation, location and source. A missing rotation is unknown; an explicitly zero rotation is invalid input reported as a violation. Coverage is always partial; an empty list does not establish compliance. The legacy entry point does not validate spin or sequence recognition and program-wide scoring consequences. The additive `validate_program_with_evidence` and profile-selected equivalent check planned regular spins; see the [contract and supported-case matrix](specs/regular-spin-validation.md). They preserve legacy jump findings and expose partial/unsupported coverage per family. Typed feature attempts remain assertions; no panel calls or levels are inferred. `tests/validation.rs` covers selected boundaries and unknowns.

## Scoring

`src/scoring.rs` covers the 12 clean singles and doubles from 1T to 2A and their q, < and << rows. Flip/Lutz ! and e base values are available, including combinations with rotation calls; scoring applies cumulative GOE limits for combined calls. Single and multiple calls use the source-defined starting caps and reductions, including the separate multiple-q rule. Inapplicable edge marks and other variants return unsupported; missing evidence remains unknown. All 187 representable regular/V spin, StSq and choreographic rows now use the common catalog, including B and numbered levels where published. The scorer accepts supplied levels and calls without inferring recognition; V alone does not identify its GOE error cause. Falls retain the -3 final-GOE ceiling. Recognized zero, ignored and no-value outcomes retain their distinctions. Timestamp-free and timed scenarios remain hypothetical. Recognized two- and three-jump combinations with toe loops or loops as later direct jumps use the sum of component base values and one GOE adjustment based on the highest component value. A supplied single-jump +REP applies 80% BV with the original applicable GOE row; a sole whole-element star returns NoValue. Supplied Euler/full-sequence units, explicit component deletion and contextual +REP with one first retained jump are now supported. All-deleted units are NoValue; execution calls on deleted jumps remain visible. Interrupted +COMBO/+SEQ BV can be calculated from supplied stars, and the approved additive execution-evidence API supports exact hypothetical GOE/totals for verified located falls and step-outs; it also constrains complete-unit scores for one full revolution and Euler step-over. Existing functions keep their incomplete results without evidence. Multiple retained jumps with +REP and other undocumented structures remain unsupported. Unverified touch-down/weight-transfer and multiple-full-revolution numerical consequences remain unsupported and are tracked in [#56](https://github.com/tizian-bitschi/skating-program-lab/issues/56) and [#58](https://github.com/tizian-bitschi/skating-program-lab/issues/58). See the [execution-evidence contract](specs/scoring-execution-evidence.md). This paragraph describes the legacy hypothetical scoring slice. Current Senior triple/quad calls, supplied multi-judge aggregation and exact selected protocol TES replay are documented in the [Senior scoring contract](specs/senior-protocol-scoring.md). Eligibility and fall deductions remain profile-selected; PCS, complete segment reconstruction and independent official adjudication remain outside numerical replay. See the [variant matrix](specs/scoring.md#implemented-scoring-details) for sources and error semantics. `tests/scoring.rs` covers source rows and timing; its five-jump selection fixture is not a legal Bronze program.

## Optimization

The original reference and Bronze solver cover the finite 1Lz domain in the [optimization specification](specs/optimization.md). The separate category-aware `optimize_jumps` API searches explicit bounded jump-only domains in all four Adult categories. It supports clean singles and doubles and explicitly listed clean two-jump combinations ending in a toe loop or loop, subject to the shared validator and table. It does not optimize sequences, Euler, other transitions, calls, or full programs. Search interruption and incomplete tie enumeration remain distinct from proofs. The unfactored table-BV objective assumes clean recognition and does not resolve program-wide scoring consequences; see [category-aware optimization](specs/optimization.md#category-aware-jump-optimizer).

The mixed-program assumed-recognition optimizer and anchored completion
maximize credited BV for the supported explicit domain (#101). They share the
manual Rule 612 credit resolver: two identical Senior advanced solos use
full credit followed by 80%, with the credit assigned in final program order.
Pumpkin's raw table sum bounds the search, and a separate flag reports
whether every optimal ordered tie was found. The evidence-based optimizer
still uses its unfactored objective. See the [proof scope and input bounds](specs/bounded-program-optimizer.md#proof-output-and-deployment-limits).

Pumpkin is enabled by default; `--no-default-features` omits both solver APIs. These optimizers do not establish complete eligibility, a global maximum or WebAssembly compatibility.

## Versioned SOV catalog (issue #45)

The [catalog reference](catalog.md) records the June 2 source hash, all 625
published value rows, numerical-column layout, notes and PCS factors. Slices A
and B were independently reviewed and merged in PRs #50 and #51. Slice C adds
all 223 Pair-specific rows and the complete numerical audit. Exact-version
`catalog_row` lookup returns immutable data with discipline, family, level, notes,
BASE, all GOE adjustments and printed page/value-row/code provenance. Unknown
notation returns `None`; unknown editions keep `UnknownVersion` without fallback.

The scorer reads single/double jumps and the 187 representable non-jump rows from
the catalog; duplicate hard-coded value arrays are removed. Issue #46 expands
spin/V, StSq and choreographic numerical scoring with an independent all-column
fixture audit. Recognition, validation and optimizer support do not expand. Pair rows are retrievable data
only. A known catalog row can still have explicitly unsupported engine behavior.

The independently reviewed source inventory is separate from production data.
Tests compare all 625 rows and their metadata, every one of the 6,875 printed
numerical cells against a separate Poppler source extraction, and 42 separately
hand-read all-GOE fixtures across every family. Existing twelve-jump fixtures and
source references remain unchanged. Inventory/numerical mutation checks reject
missing codes/columns, duplicates, incorrect metadata/values, and edition/hash
mismatches. `tools/verify_sov_source.py` reproduces the source fixture from the
exact pinned PDF; neither it nor a PDF parser is used at runtime.

There are no unresolved extraction mismatches. The source audit and visual
cross-check limits, including the lack of independent human cell-by-cell review,
are explicit in the catalog reference. Notes and PCS factors are recorded data,
not added scoring/eligibility rules or an implemented PCS calculation.

## Versioned rule profiles (#47)

`ProfileRegistry::adult` registers the four current Adult definitions.
`validate_profile_program`, `score_profile_program` and feature-gated
`optimize_profile_jumps` select exact identities and share the existing engines.
The old APIs and struct literals remain supported. `tests/profiles.rs` covers
source-backed Adult eligibility, adapter equivalence, identity/version errors,
invalid sources and policies, override ordering/conflicts, unsupported scope and
mechanisms, typed family/repetition policy, scoring and a synthetic extension.
Existing validation, scoring, catalog and optimizer fixtures remain unchanged.

Registry validation checks citation metadata, not source authenticity. The
registry's unimplemented feature/level policy mechanisms block selection;
separate source-bound spin and mixed-program policies support the bounded
evidence checks described below. Numerical scoring retains partial scope and
does not establish category eligibility. The profile jump optimizer retains
its finite jump domain and maximum five slots; the separate mixed-program
optimizer supports eleven Senior slots and ten slots for other profiles. Senior
Women/Men profiles are now available; Junior and national profiles remain absent. See the [ADR](adr/0002-rule-profiles-proposal.md).

Review regression tests additionally reject contradictory Axel/sequence/family
requirements, missing or mismatched SOV source editions and unsupported scoring
binding overrides. Optimizer sources contain each exact citation once, in stable
order; distinct SOV row locations remain distinct. ProfileRule is shared by
registry and validator. Scoring binding overrides are explicitly unsupported;
new versioned scoring policies use explicit policy citations.

## Bounded mixed-program scenarios (issue 49)

`optimize_program` and `validate_supported_program` add supported mixed-element planning with explicit recognition/qualification assumptions and partial coverage. See the [supported contract and limitations](specs/bounded-program-optimizer.md). Difficult variation categories, Euler consequences, automatic recognition, Pair rules and full compliance remain outside the model.

## Candidate generation (#65)

`generate_program_candidates` provides a bounded deterministic supported domain
for the existing mixed optimizer. Built-in profiles share one generator and the
existing validators/scorer. Automatic jump and Basic-spin definitions, explicit
qualified spin/movement templates, semantic personal filters, manual subsets,
source audit and separate domain-cap metadata are covered by
`tests/candidate_generation.rs`. See the [generation contract](specs/bounded-program-optimizer.md#finite-candidate-generation-65).
Unsupported posture/features and physical transitions remain omitted with reasons;
full-program rules and recognition uncertainty remain unchanged. Candidate-domain
completeness is distinct from optimizer search/tie completeness.

Review regressions additionally cover per-sequence explicit direct-step evidence,
spin position/entry filters on automatic and manual combinations, manual-input
validation independent of automatic caps, normalized unordered step feature
sets, and typed finding scope shared by generator and optimizer preflight.

## Product workflow acceptance (#68)

The [four-function acceptance matrix and measured bounds](milestone-68.md) maps
manual assessment, legality, generated optimization and anchored completion to
shared-entry-point fixtures. Independent tiny-domain oracles compare maximum BV
and every ordered optimum, including distinct anchor placements. All returned
programs are reassessed and revalidated. Objective proof, generation completeness,
tie enumeration and regulatory coverage remain separate. Broad six-slot native
domains do not produce incumbents within the measured 250 ms budget; this is a
recorded performance limit, not evidence of infeasibility or global optimality.

## Complete incremental planning (#140–#143)

`PlanningSearchSession` provides separate maximum proof and uncapped incremental
collection, cooperative pause/resume, retained confirmed results and explicit
exhaustion. Existing search functions retain at most five results. Session inputs
and registry selection stay fixed for its lifetime; no persistence or background
worker is supplied. The six built-in profiles and registered synthetic policies
exercise unrestricted, required-only, restricted-only and combined workflows.
Neutral ignored movements follow policy mechanisms, while required ignored
occurrences and counted zero-value intentions retain their identity. Shared fresh
manual assessment checks each delivered composition. The
[acceptance report](planning-session-acceptance.md) records tests, limitations and
native proof/collection measurements. Planned-construction compliance excludes
actual execution and duration.
