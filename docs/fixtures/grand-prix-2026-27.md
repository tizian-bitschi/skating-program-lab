# Grand Prix 2026/27 protocol observations (#76)

Retrieved and visually checked 2026-11-04. These are factual observations of
published panel recognition and scores. They do not establish Senior legality,
motion evidence, feature attempts, or optimality. No production scorer or validator
generated the expected values. SOV remains Communication 2786, updated June 2,
2026; the existing 625-row SOV fixtures and Adult tests are unchanged.

## Source editions

The official [France index](https://results.isu.org/results/season2627/gpfra2026/)
and [Canada index](https://results.isu.org/results/season2627/gpcan2026/) each link
Men/Women **Free Skating / Judges Scores (pdf)**. The exact report links, segment
identifiers, printed creation times, retrieval date, page counts and downloaded
byte SHA-256 digests are in
[`sources.tsv`](../../tests/fixtures/grand-prix-2026-27/sources.tsv).
All four PDFs have six pages. The source key combines the event and gender;
`FSKMSINGLES-----------FNL-000100--` and
`FSKWSINGLES-----------FNL-000100--` identify the exact final segments.
The Canada index date range begins October 29; its PDF banner begins October 30.
Both identify the same Kelowna event. Printed report times are transcribed as
printed, without inferring a timezone or an athlete timestamp.

| Source | Printed edition | SHA-256 |
| --- | --- | --- |
| gpfra2026-M | 25.10.2026 15:03 | `ff61bf7603ce88a94d1794b1caaff09869eb524995d7e6a7cde95f52b041be78` |
| gpfra2026-W | 24.10.2026 21:22 | `7501c3305802e0ef42a56e616d2dd709d4bd844199346746d18284e789917c9b` |
| gpcan2026-M | 31.10.2026 18:34 | `f95e8e2ee7f820780ab76d9b15ef326d9620be3e10d9fb1d856b79799023a961` |
| gpcan2026-W | 01.11.2026 13:55 | `0d475f51aaff8001213cb44bac462db41d936f513bac1cc11aa8fc89e44345a1` |

To refresh, download each exact URL and compare its byte digest before extracting
anything. A changed digest requires a new edition and a complete source review;
never silently overwrite this snapshot. Keep copyright PDFs and rendered pages
locally outside the repository. Only derived selected factual data is committed.

## Selection and coverage inventory

Twelve distinct skaters, three per event/gender, each with all eleven ordered
element rows, all nine published judge columns, all three PCS rows and every
deduction row. Page and rank refer to the printed report, not overall standings.

| Source | Page / rank / start | Skater | Deliberate coverage |
| --- | --- | --- | --- |
| gpfra2026-M | 1 / 1 / 12 | Adam SIAO HIM FA | clean quad combination, Euler/sequence, x, late start |
| gpfra2026-M | 3 / 5 / 11 | Aleksandr SELEVKO | q, repeated jump types, `2A*` zero value, nine absent grades |
| gpfra2026-M | 6 / 11 / 9 | Mikhail SHAIDOROV | `4F!<`, `4T+REP`, x, two falls |
| gpfra2026-W | 1 / 1 / 10 | Ami NAKAI | triple Axel, !, clean combination/sequence, level 4 |
| gpfra2026-W | 3 / 5 / 12 | Yuna AOKI | FCSp3, repeated 2Lz, component `2T<`, fall |
| gpfra2026-W | 6 / 12 / 3 | Seojin YOUN | `3Lz<<+REP`, downgraded component, four falls / -6.00 |
| gpcan2026-M | 1 / 1 / 8 | Mihhail SELEVKO | 3A< inside sequence, q, CCoSp3 |
| gpcan2026-M | 4 / 7 / 5 | Georgii RESHTENKO | CCoSp2, FSSp3, StSq2, underrotated component |
| gpcan2026-M | 6 / 11 / 7 | Hyungyeom KIM | `4T<+REP`, `q,q`, `3A<*`, absent grades |
| gpcan2026-W | 1 / 1 / 12 | Mao SHIMADA | triple Axel, quad <, component !, clean level 4 rows |
| gpcan2026-W | 1 / 2 / 10 | Niina PETROKINA | CCoSp4V, repeated 3F!, x sequence |
| gpcan2026-W | 5 / 9 / 3 | Fee Ann LANDRY | sequence-component < and fall, separate time/fall deductions |

This is a selected sample, not a census or a collection of completely clean
programs. Basic/level-1 regular spins, unknown recognition and a starred component
inside an otherwise valued compound element are absent from this selection; no
fixture invents those observations. Whole starred elements occur twice. Every
selected Ref. column is blank. No grade is fabricated for a printed `-`.

## Fixture and provenance contract (v1)

UTF-8 TSV files with headers, one physical line per record, no quoting/escaping
needed for these values. Decimal strings retain exactly two printed decimal
places and their signs. Empty cells mean an unprinted field; `-` means the
literal printed missing-grade marker. Do not replace missing grades with zero.

- `programs.tsv`: name, nation, starting number, segment score, TES, factored PCS,
  signed deduction total, printed BASE subtotal and element-score footer.
- `elements.tsv`: ordered row, verbatim recognized code, verbatim Info column
  (including `F`, commas and calls), credited BASE, printed `x`, GOE adjustment,
  J1 through J9 in order, blank Ref., element score, and support classification.
  Component-specific marks remain in their original positions in `code`, e.g.
  `3Aq+3Tq` and `3F+2T<`; Info alone cannot locate them. `SEQ` and `REP` remain
  verbatim suffixes. The printed BASE already includes x/repetition reductions;
  do not apply them again.
- `components.tsv`: ordinal, printed PCS label, factor, J1-J9, Ref. and result.
- `deductions.tsv`: separate reason, signed amount and printed parenthesized
  count without parentheses. Zero-deduction programs have no deduction records.
- `reviewed-source.tsv`: separate source transcription in column order, with
  pipe-separated printed fields; keyed by source/page/rank/kind/row. Programs
  use implicit row zero. It excludes the editorial support label.

Every field inherits `source` (exact PDF URL and digest), printed `page`, `rank`,
record kind, row and named TSV column. Thus `gpcan2026-M / 6 / 11 / element / 7 /
j1` points to the printed `-` in Hyungyeom KIM's seventh element. Program identity
also includes name, nation and starting number; rank alone is not a global ID.

`supported` describes a bounded numerical recognized-row representation only.
`requires #78/#77` labels rows containing triple/quad jumps, including those
inside compounds. The other allowed classifications are `recognition-unknown`
and `outside-scope`; there are no unknown element codes in this selected sample.
All Senior program legality and PCS/deduction adjudication are **outside-scope**
for the current Adult engine. Even `supported` rows do not assert Senior legality
or official nine-judge panel-score support. No timing, observed movement or spin
feature evidence is inferred from a protocol abbreviation.

## Independent cross-check and maintenance

The fixture transcription used native PDF text-stream order. A second pass read
coordinate-ordered table columns into the source manifest, independently of the
fixture rows and production library. The selected rendered pages were then
visually inspected for identity, code/Info alignment, BASE/x/GOE, J1-J9, Ref.,
scores, PCS and deduction columns: France M/W pages 1, 3, 6; Canada M pages 1, 4,
6; Canada W pages 1, 5. In particular, the x on a zero BASE, nine dashes on a
starred row, two separate deduction reasons, `!,<` and `q,q` were checked in the
renderings. This is an agent source cross-check, not a claim of human domain
certification. The external PR review is separate.

`cargo test --test grand_prix_fixtures` runs without network/PDF parsing or scorer
calls. It compares every factual cell to the reviewed manifest, detects missing,
duplicate and orphan locations, checks exact decimal syntax, eleven contiguous
elements and three PCS rows per program, and verifies BASE + GOE, both printed
subtotals, TES and segment accounting. Signed deductions are added to TES + PCS
only when checking the segment score; TES remains the element-score sum.
A deliberate grade-column corruption test verifies a location/field diagnostic.
Complete ordered headers are pinned for all six TSV files; test logic accesses
fields by name. A separate reviewed code matrix checks every support label,
including triples/quads inside marked compounds. Mutation tests reject swapped
judge headers and incorrect support labels.

PCS accounting independently multiplies each printed component result by its
factor, rounds each component to hundredths, then sums and compares with the
program's factored PCS. Source-grounded rounding cases include Hyungyeom KIM
(Canada M p. 6/rank 11: 20.35 + 21.05 + 21.51 = 62.91), Aleksandr SELEVKO
(France M p. 3/rank 5: 81.36) and Seojin YOUN (France W p. 6/rank 12: 43.78).
Rounding only after summing gives one-cent differences in these cases; a
regression test rejects that error with a skater/page diagnostic. Individual PCS
judge marks are preserved and parity-checked, without rejudging their aggregate.
The tests do not rejudge GOE or validate Senior programs.
