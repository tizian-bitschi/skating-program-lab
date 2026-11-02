# Implementation status

The library provides partial coverage. These tests cover selected cases, not every domain rule. See [rule sources](rule-sources.md) for editions and open questions.

## Domain

`src/lib.rs`, `src/plan.rs` and `src/execution.rs` model version and source identities, ordered plans, observations and supplied panel outcomes. `tests/domain.rs`, `tests/plan.rs` and `tests/execution.rs` cover selected representations and errors.

Jump transitions are distinct from listed jumps. Regular spins distinguish flying and non-flying entries and preserve the order of positions and foot changes; choreographic spins are separate. Feature attempts are descriptions attached to positions, not recognized features. An unspecified feature list differs from an explicitly empty one. Planned jump type and rotations may be missing. Observed movement is free text and revolutions are whole counts. Indices refer to caller-provided lists and are not validated. Source applicability and metadata consistency are not validated. Recognition notation and constructor limitations are in the [scoring specification](specs/scoring.md#recognition-representation).

## Validation

`src/validation.rs` checks selected Bronze jump eligibility, element count, combination count and structure, Euler prohibition and repetitions. Missing details yield unknown findings; other categories and seasons are unsupported. Findings carry a rule identifier, explanation, location and source. Coverage is always partial; an empty list does not establish compliance. Spin and sequence recognition and program-wide scoring consequences are not validated. `tests/validation.rs` covers selected boundaries and unknowns.

## Scoring

`src/scoring.rs` looks up selected 1Lz rows, 1A<<, CSSp1 and ChSp1. It scores unmarked 1Lz, CSSp1 and ChSp1 and the two supported zero-valued downgrade rows for one hypothetical judge. Timestamp-free planning scenarios support an optional hypothetical second-half boundary, without claiming exact TES. Program scenarios include separate Bronze fall deductions and category time deductions under supplied timing assumptions. Unsupported or unknown elements prevent exact TES; ignored and no-value outcomes remain distinct. Other rows, combinations, most marked scores, other categories' fall deductions, PCS and full official results are unsupported. See [scoring details](specs/scoring.md#implemented-scoring-details) for timing, sources and limits. `tests/scoring.rs` covers selected rows and timing scenarios; its five-jump selection fixture is not a legal Bronze program.

## Optimization

The reference and Pumpkin solver cover only the finite Bronze 1Lz domain in the [optimization specification](specs/optimization.md). The solver is enabled by default; `--no-default-features` omits Pumpkin. `src/optimizer.rs`, `src/optimizer/pumpkin.rs` and `tests/optimizer.rs` cover this domain, element-count bounds and result limits. These do not establish complete eligibility, a global maximum or WebAssembly compatibility.
