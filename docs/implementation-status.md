# Implementation status

The library provides partial coverage. These tests cover selected cases, not every domain rule. See [rule sources](rule-sources.md) for editions and open questions.

## Domain

`src/lib.rs`, `src/plan.rs` and `src/execution.rs` model version and source identities, ordered plans, observations and supplied panel outcomes. `tests/domain.rs`, `tests/plan.rs` and `tests/execution.rs` cover selected representations and errors.

Jump transitions are distinct from listed jumps. Regular spins distinguish flying and non-flying entries and preserve the order of positions and foot changes; choreographic spins are separate. Feature attempts are descriptions attached to positions, not recognized features. An unspecified feature list differs from an explicitly empty one. Planned jump type and rotations may be missing. Observed movement is free text and revolutions are whole counts. Indices refer to caller-provided lists and are not validated. Source applicability and metadata consistency are not validated. Recognition notation and constructor limitations are in the [scoring specification](specs/scoring.md#recognition-representation).

## Validation

`src/validation.rs` checks jump eligibility, element count, combinations, sequences, Euler and listed-jump repetitions for Bronze, Silver, Gold and Masters. Masters requires an Axel-type jump. Transitions outside the documented direct subset, direct sequence steps, and Euler footwork or panel consequences remain unknown; other seasons are unsupported. Findings carry a rule identifier, explanation, location and source. A missing rotation is unknown; an explicitly zero rotation is invalid input reported as a violation. Coverage is always partial; an empty list does not establish compliance. Spin and sequence recognition and program-wide scoring consequences are not validated. `tests/validation.rs` covers selected boundaries and unknowns.

## Scoring

`src/scoring.rs` covers the 12 clean singles and doubles from 1T to 2A and their q, < and << rows. Flip/Lutz ! and e base values are available, including combinations with rotation calls; scoring combined calls remains unsupported. Single calls are scored with applicable final-GOE consistency limits. Inapplicable edge marks and other variants return unsupported; missing evidence remains unknown. All 167 representable regular/V spin, StSq and choreographic rows now use the common catalog, including B and numbered levels where published. The scorer accepts supplied levels and calls without inferring recognition; V alone does not identify its GOE error cause. Falls retain the -3 final-GOE ceiling. Recognized zero, ignored and no-value outcomes retain their distinctions. Timestamp-free and timed scenarios remain hypothetical. Recognized two-jump combinations with a toe loop or loop as the second jump use the sum of component base values and one GOE adjustment based on the highest component value. Three-jump, Euler, sequence, repetition and starred consequences remain unsupported. Category eligibility, triple and quadruple jumps, other categories' fall deductions, PCS and official results remain outside this scoring slice. See the [variant matrix](specs/scoring.md#implemented-scoring-details) for sources and error semantics. `tests/scoring.rs` covers source rows and timing; its five-jump selection fixture is not a legal Bronze program.

## Optimization

The original reference and Bronze solver cover the finite 1Lz domain in the [optimization specification](specs/optimization.md). The separate category-aware `optimize_jumps` API searches explicit bounded jump-only domains in all four Adult categories. It supports clean singles and doubles and explicitly listed clean two-jump combinations ending in a toe loop or loop, subject to the shared validator and table. It does not optimize sequences, Euler, other transitions, calls, or full programs. Search interruption and incomplete tie enumeration remain distinct from proofs. The unfactored table-BV objective assumes clean recognition and does not resolve program-wide scoring consequences; see [category-aware optimization](specs/optimization.md#category-aware-jump-optimizer).

Pumpkin is enabled by default; `--no-default-features` omits both solver APIs. These optimizers do not establish complete eligibility, a global maximum or WebAssembly compatibility.

## Versioned SOV catalog (issue #45)

The [catalog reference](catalog.md) records the June 2 source hash, all 625
published value rows, numerical-column layout, notes and PCS factors. Slices A
and B were independently reviewed and merged in PRs #50 and #51. Slice C adds
all 223 Pair-specific rows and the complete numerical audit. Exact-version
`catalog_row` lookup returns immutable data with discipline, family, level, notes,
BASE, all GOE adjustments and printed page/value-row/code provenance. Unknown
notation returns `None`; unknown editions keep `UnknownVersion` without fallback.

The scorer reads single/double jumps and the 167 representable non-jump rows from
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
