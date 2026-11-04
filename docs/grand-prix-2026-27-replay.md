# Official technical-score replay (#80)

The offline harness in `tests/official_replay.rs` runs the twelve curated
[#76 reports](fixtures/grand-prix-2026-27.md) through the public #78
`score_protocol_program` API with the matching Women/Men Senior profile.
Run `cargo test --test official_replay -- --nocapture` for program summaries.
The test runtime reads committed TSV data only. Fixture schema and strict
decimal helpers are shared with `grand_prix_fixtures` in `tests/support/fixture.rs`.

## Numerical results

All 132 ordered rows match printed credited BASE, numerical panel GOE and
element score exactly to hundredths. All twelve printed TES totals match.
The two wholly starred rows have `NoValue`, missing judge marks and a known
zero contribution. They remain separate from unsupported recognition.

| Event / group | Skater | PDF page / rank | Status | Published / replay TES | Deductions |
| --- | --- | --- | --- | --- | --- |
| France Men | Adam SIAO HIM FA | 1 / 1 | exact | 91.94 / 91.94 | -1.00 |
| France Men | Aleksandr SELEVKO | 3 / 5 | exact | 63.13 / 63.13 | 0.00 |
| France Men | Mikhail SHAIDOROV | 6 / 11 | exact | 58.84 / 58.84 | -2.00 |
| France Women | Ami NAKAI | 1 / 1 | exact | 80.43 / 80.43 | 0.00 |
| France Women | Yuna AOKI | 3 / 5 | exact | 57.31 / 57.31 | -1.00 |
| France Women | Seojin YOUN | 6 / 12 | exact | 37.28 / 37.28 | -6.00 |
| Canada Men | Mihhail SELEVKO | 1 / 1 | exact | 88.56 / 88.56 | 0.00 |
| Canada Men | Georgii RESHTENKO | 4 / 7 | exact | 75.21 / 75.21 | 0.00 |
| Canada Men | Hyungyeom KIM | 6 / 11 | exact | 59.52 / 59.52 | -1.00 |
| Canada Women | Mao SHIMADA | 1 / 1 | exact | 85.65 / 85.65 | 0.00 |
| Canada Women | Niina PETROKINA | 1 / 2 | exact | 73.93 / 73.93 | 0.00 |
| Canada Women | Fee Ann LANDRY | 5 / 9 | exact | 60.77 / 60.77 | -2.00 |

Each group has three exact programs and no partial programs. TES is the sum
of technical element scores. Deductions are separately summed from published
deduction rows; no deduction or PCS enters TES. No PCS/TSS reproduction is claimed.

## Source and comparison contract

`sources.tsv` identifies each exact PDF URL, SHA-256, segment and edition.
Each row report names the event, group, skater, PDF page, rank and element
ordinal. The original code and canonical supplied recognition stay separate.
The shared bounded test adapter is in `tests/support/protocol_recognition.rs`;
it supplies calls and levels, not execution observations or inferred intent.

The independent `reviewed-source.tsv` transcription checks every printed
element field, including code, Info, ordinal, x, all nine judge positions,
GOE, BASE and score. A changed grade that leaves the trimmed mean unchanged
still prevents an exact status. Row comparison uses the printed source ordinal and separately verifies its
strict chronological order. Swapping complete records prevents exact status;
a missing row preserves verified contributions from the remaining source rows.

The independent June 2 SOV extraction
`tests/fixtures/isu-2786-2026-06-02-source-values.tsv` supplies each recognized
component's table BASE, page and value-row number. Component sums exclude
REP and x; the credited BASE is compared separately with the protocol.
Call reductions and downgraded rows use their explicitly published SOV entries.
Whole starred rows retain `NoValue` instead of inventing a zero-valued SOV row.
The existing catalog tests independently verify all source table columns.

Rule 353.1(b–h), June 12 Sports Rules pp. 9–10, defines numerical panel
aggregation and rounding. Each judge grade goes through its SOV numerical
column before removing one low and one high score and averaging seven marks.
The existing `panel_scoring` tests cover rounding ties and cases where grade
averaging gives the wrong answer. Communication 2786 p. 14 remarks 4–5 and
the Technical Panel Handbook pp. 27–33 supply compound, deletion and REP
context. See the [#78 numerical source review](specs/senior-protocol-scoring.md).
France Women p. 6/rank 12 `3Lz<<+REP` tests the distinction between table
BASE 2.10, REP/x credited BASE 1.85 and GOE -1.05. France Men p. 1/rank 1
contains uncredited `4T+2T` and credited `4T`, plus the four-component
`3A+Eu+2S+2A+SEQ`. Canada Women p. 1/rank 2 supplies `CCoSp4V`.

## Discrepancy and partial totals

The reusable `replay` function returns structured program and row reports.
Each difference records the field, expected and actual values, and signed
arithmetic difference for numerical scorer comparisons. Reports retain the
raw/canonical code, SOV source rows, outcome and missing fact/rule or scoring
error. `exact_tes` is absent if any row is unsupported, any source field differs,
the row count changes, or the published TES differs. `verified_subtotal` sums
only individually exact rows; unresolved/mismatched rows remain listed.
Unknown and unsupported SOV tokens become unknown recognition with separate
expected source amounts and absent actual BV/GOE/score values. Explicit row
states are Exact, Mismatch, Unsupported and Unreproducible. Extra rows and
missing reference positions produce structured gaps while preserving all
individually verified subtotal contributions.
Missing judge positions remain missing, never zero or a smaller panel.

Mutation tests alter published GOE, individual grades, code (including Pair-only
`ChPLi1`), printed x, chronological order, missing/extra rows and missing manifest
references. A `j8 = 4` mutation leaves the trimmed numerical GOE and score
unchanged while source verification prevents exact status. Each prevents exact
status and retains the skater/row location. Expected scores come from the
source fixtures, never from production scorer output.

## Manual assessment and remaining limits

Ami NAKAI's complete eleven-row France report also passes through
`assess_manual_program` and `assess_program_legality`. Each supplied recognition
has its exact PDF edition and element location. This is a deliberately
incomplete hypothetical family envelope: jump plan slots have unknown kind and
rotation, regular spins have unknown entry/steps/features, and step/choreographic
movement evidence is absent. The placeholder solo slots express no claim about
the athlete's intended compound choreography. Supplied recognition remains
separate in scenarios. No scalar final GOE, fall, timing boundary or execution
evidence is fabricated. Both entry points return the same shared validation;
assessment is `Indeterminate`. The replay adapter preserves raw API validation
and projects placeholder-dependent compound-structure/transition applicability
to source-linked `MissingEvidence` coverage. The solo placeholders establish no
claim that compound rules are inapplicable to the recognized program. Tests
pin both rule coverage states and preserve the original regulatory references.
This report adaptation lives in `tests/support/manual_replay.rs`; the production
legality API is unchanged.

Exact **numerical replay of supplied recognition** does not establish independent
timing, recognition eligibility or execution legality. Published x is supplied
credit, not a reconstructed jump start time. Unknown calls, component stars,
ignored entries and unsupported execution reductions beyond the observed sample
are not added as fictitious protocol observations. Existing synthetic API tests
cover NoValue/Ignored and component-star distinctions. #56 and #58 remain open.
Adult fixtures, table editions, public interfaces and CI configuration are unchanged.
