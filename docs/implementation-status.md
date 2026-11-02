# Implementation status

The library provides partial coverage. These tests cover selected cases, not every domain rule. See [rule sources](rule-sources.md) for editions and open questions.

## Domain

`src/lib.rs`, `src/plan.rs` and `src/execution.rs` model version and source identities, ordered plans, observations and supplied panel outcomes. `tests/domain.rs`, `tests/plan.rs` and `tests/execution.rs` cover selected representations and errors.

Jump transitions are distinct from listed jumps. Regular spins distinguish flying and non-flying entries and preserve the order of positions and foot changes; choreographic spins are separate. Feature attempts are descriptions attached to positions, not recognized features. An unspecified feature list differs from an explicitly empty one. Planned jump type and rotations may be missing. Observed movement is free text and revolutions are whole counts. Indices refer to caller-provided lists and are not validated. Source applicability and metadata consistency are not validated. Recognition notation and constructor limitations are in the [scoring specification](specs/scoring.md#recognition-representation).

## Validation

`src/validation.rs` checks jump eligibility, element count, combinations, sequences, Euler and listed-jump repetitions for Bronze, Silver, Gold and Masters. Masters requires an Axel-type jump. Transitions outside the documented direct subset, direct sequence steps, and Euler footwork or panel consequences remain unknown; other seasons are unsupported. Findings carry a rule identifier, explanation, location and source. A missing rotation is unknown; an explicitly zero rotation is invalid input reported as a violation. Coverage is always partial; an empty list does not establish compliance. Spin and sequence recognition and program-wide scoring consequences are not validated. `tests/validation.rs` covers selected boundaries and unknowns.

## Scoring

`src/scoring.rs` covers the 12 clean singles and doubles from 1T to 2A and their q, < and << rows. Flip/Lutz ! and e base values are available, including combinations with rotation calls; scoring combined calls remains unsupported. Single calls are scored with applicable final-GOE consistency limits. Inapplicable edge marks and other variants return unsupported; missing evidence remains unknown. CSSp1, ChSp1, recognized zero, ignored and no-value outcomes retain their distinctions. Timestamp-free and timed scenarios remain hypothetical. Recognized two-jump combinations with a toe loop or loop as the second jump use the sum of component base values and one GOE adjustment based on the highest component value. Three-jump, Euler, sequence, repetition and starred consequences remain unsupported. Category eligibility, triple and quadruple jumps, other categories' fall deductions, PCS and official results remain outside this scoring slice. See the [variant matrix](specs/scoring.md#implemented-scoring-details) for sources and error semantics. `tests/scoring.rs` covers source rows and timing; its five-jump selection fixture is not a legal Bronze program.

## Optimization

The original reference and Bronze solver cover the finite 1Lz domain in the [optimization specification](specs/optimization.md). The separate category-aware `optimize_jumps` API searches explicit bounded jump-only domains in all four Adult categories. It supports clean singles and doubles and explicitly listed clean two-jump combinations ending in a toe loop or loop, subject to the shared validator and table. It does not optimize sequences, Euler, other transitions, calls, or full programs. Search interruption and incomplete tie enumeration remain distinct from proofs. The unfactored table-BV objective assumes clean recognition and does not resolve program-wide scoring consequences; see [category-aware optimization](specs/optimization.md#category-aware-jump-optimizer).

Pumpkin is enabled by default; `--no-default-features` omits both solver APIs. These optimizers do not establish complete eligibility, a global maximum or WebAssembly compatibility.
