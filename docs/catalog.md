# ISU 2786 catalog

## Edition and source inventory

The catalog contract selects `ScoringTableId::Isu2786June2_2026`. Its source is
[ISU Communication 2786, updated June 2, 2026](https://isu-d8g8b4b7ece7aphs.a03.azurefd.net/isudamcontainer/CMS/Corporate-Site/Governance/Transparency/ISU-Communications/2786-SinglesPairs-SOV-Update-260602-1780481099-6311.pdf),
valid from July 1, 2026. Downloaded PDF SHA-256:
`980ca3f689c280196f1fe4e32843e97010116c8b624eb7871c9f8850561613b7`.
The cover states June 2; the internal PDF title still names May 8. The internal
title must not select the edition. No May values or fallback are used.

`tests/fixtures/isu-2786-2026-06-02-manifest.tsv` inventories each printed value
row independently of production data. Page is the printed page; row is the
one-based ordinal of value rows on that page, excluding headers and separators.
Each row names its section, exact code, discipline, family, level, numeric-column
count and applicable p. 14 note references. `render-cross-checked` records the
implementation author's check against rendered source pages; it does not claim
independent review approval. The external PR review must check this inventory.

`tests/fixtures/isu-2786-2026-06-02-source-codes.txt` is a separately maintained,
compact reference transcribed from the printed code columns, grouped by page,
family and discipline. It is not generated from the manifest or production
catalog. Tests compare all 625 codes and their order/placement, then check level
and note consistency using family-aware marker boundaries. Mutation tests prove
that replacing `StSq4` with `StSq5`, corrupting a note, or changing level, family
or discipline fails even when totals remain unchanged.

| Printed page | Value rows | Sections |
| --- | ---: | --- |
| 1 | 0 | Cover and edition |
| 2 | 55 | Jumps, including Euler |
| 3 | 52 | Jumps and Advanced Novice bonus codes |
| 4 | 54 | Jumps, including 3A/3Ab/3Aq/3Aqb |
| 5 | 54 | Jumps, including quints |
| 6 | 50 | Spins in one position; flying spins |
| 7 | 55 | Flying spins; spins with change of foot |
| 8 | 55 | Change-of-foot spins; combinations without change of foot |
| 9 | 45 | Combinations with change of foot; steps; lift groups 1-4 |
| 10 | 45 | Lift groups 5S/5T/5B/5A/5R; twists |
| 11 | 52 | Twists; throws |
| 12 | 54 | Throws |
| 13 | 40 | Throws; death spirals and bonus codes; pivot; pair spins |
| 14 | 14 | Pair spin combinations; four choreographic elements |
| Total | 625 | 14 families, 625 distinct exact codes |

The source table has eleven numerical columns: GOE -5..-1, BASE, and GOE +1..+5.
The Rust contract stores BASE separately and all eleven GOE adjustments, with
zero at index 5 for GOE 0. Integer hundredths preserve the printed values rather
than recalculating them from percentages. Extraction misses four graphic-only
code labels on p. 4 (`3A`, `3Ab`, `3Aq`, `3Aqb`); they were restored from the
rendered source. Some cells have graphic overlays, so text alone is insufficient.
No unresolved inventory ambiguity remains; numerical parity is checked in the
following data slices, not established by the inventory alone.

## Published notes and other metadata

`CatalogNote` references the numbered remarks and legend on p. 14. These are data
annotations, not implemented eligibility, recognition or scoring rules:

- 1: regular GOE increments use 10% of applicable BV.
- 2: choreographic increments use a higher scale, recorded explicitly per row.
- 3: `<`, `e` and `V` use reduced BV for GOE; `<<` identifies downgraded rows.
- 4: combination/sequence difficulty uses the applicable reduced BV.
- 5: `+Rep` reduces BV to 80%, with GOE based on original BV.
- 6: quints are excluded from Short Programs and from combinations/sequences in
  Free Skating; repetition limits and the stated 80% solo-repeat consequence are
  data-only. No unlisted `+Rep` row is manufactured.
- Legend: `b` is an Advanced Novice Singles/Pairs bonus marker, `q` quarter,
  `<` under-rotation, `<<` downgrade (also twists), `!` unclear edge, `e` wrong
  edge, and `V` reduced spin value. Published bonus rows retain their printed BV;
  no bonus amount is inferred or added.

Remark 3 is attached only to the explicitly named single `<`, `e` or `V` markers.
A downgrade-only `<<` row references the legend, not remark 3. A combined
`e<<` row still references remark 3 because of its `e`. This records the source's
wording without inferring scoring behavior. Markers are parsed after the element
name; the `q` inside `StSq`/`ChSq1` is not a quarter call. These unmarked sequence
codes carry no legend reference.

Section 2 on p. 14 separately publishes Senior/Junior program component factors:
Men SP 1.67 / FS 3.33; Women and Pairs SP 1.33 / FS 2.67. These are documented
source metadata, not SOV rows or a PCS implementation.

## Delivery and support

The user approved the public contract and three sequential PRs on November 2,
2026. A introduces the complete source inventory and typed `CatalogRow` contract.
B adds shared jump/spin/step/choreographic data and exact-version `catalog_row`
lookup, then migrates existing scorer rows without extending engine behavior.
C adds Pair data and completes the independent fixture/completeness audit. Issue
#45 stays open until C is reviewed, checked and merged.

A was merged as [PR #50](https://github.com/tizian-bitschi/skating-program-lab/pull/50)
after [independent GPT-6 review](https://chatgpt.com/c/6ae8b660-63b4-83ed-b2c7-524e78d13735)
returned APPROVE for commit `58b5a21`.

B embeds 402 shared rows: 215 jumps (including Euler), 180 spins, five step
sequences and two choreographic rows. `catalog_row(table, exact_code)` returns an
immutable static row or `None`, with no parsing, category filtering or fallback.
The complete 625-row source manifest is retained. C adds the 223 Pair-specific
rows; all 625 published rows are now available through the same lookup.

The old hard-coded `jump_row` values and CSSp1/ChSp1 arrays are removed. The scorer
reads the existing single/double subset and these two spin rows from the shared
catalog. Its recognition limits, unsupported errors, final-GOE restrictions and
legacy source descriptions are preserved. Catalog presence does not extend engine
support for triples, quints, other spins or steps.

Tests compare every shared row's source/metadata/notes to the independently
reviewed manifest and 23 manually source-read numerical fixtures cover all five
shared families and all GOE columns, including several levels and reduced/bonus
variants. Existing twelve-jump all-column scoring fixtures remain unchanged.
An internal cardinality/uniqueness test rejects extra or duplicate production
codes. Page/row boundaries independently validate every manifest section name;
this exposed and corrected the CSp and FCSp section labels on p. 6 (15 rows,
without changing any values). Eight corruption cases cover the manifest checker.

Manifest updates require source review; never regenerate expected coverage from
production rows. Printed numerical values are transcribed directly, including
rounded V rows, rather than recomputed using percentage arithmetic.

## Complete dataset and numerical audit (C)

B was merged as [PR #51](https://github.com/tizian-bitschi/skating-program-lab/pull/51)
after [independent GPT-6 review](https://chatgpt.com/c/6ae8bcd6-1e10-83ed-964f-20065978704b)
returned APPROVE for commit `12741ab`.

C adds 45 lifts, 40 twists, 92 throws, 28 death spirals, one pivot figure, five
pair spins, ten pair spin combinations, one choreographic pair spin and one
choreographic pair lift. These 223 rows complete all 625 exact codes in the
source manifest. Pair rows remain data only: the recognition constructor does not
accept their codes and no Pair scorer, validator or optimizer is introduced.

`tests/fixtures/isu-2786-2026-06-02-source-values.tsv` records every one of the
6,875 printed numerical cells in source-column order: -5..-1, BASE, +1..+5.
It comes from a separate Poppler 22.02.0 extraction, not the PyMuPDF production
transcription, production Rust arrays or their generator input. Code/page/row
placement comes from the independently reviewed code inventory; the four
graphic-only 3A labels are explicit exceptions, never inferred from a BV.

The audit compares every production BASE/GOE cell and precise row placement with
this fixture and verifies that its code set equals the full manifest. It checks
document, edition and source hash. Six corruption cases exercise changed values,
missing GOE cells, duplicate codes, wrong editions and wrong hashes. A private
cardinality/uniqueness check detects extra or duplicate production rows. The 42
separately hand-read fixtures cover all 14 families, multiple levels, technical
calls, bonus and reduced-value cases, and the entire GOE range. The existing
single/double jump fixtures and source references are unchanged.

Reproduce the full source audit using Python's standard library and Poppler
(`pdftotext` on PATH), outside the runtime library:

```sh
python tools/verify_sov_source.py /path/to/2786-SinglesPairs-SOV-Update-260602.pdf
python -B tools/test_verify_sov_source.py
cargo test --test catalog --test catalog_manifest
```

The verifier rejects any PDF with a different SHA-256 before extraction. It checks
all page counts and extracted code labels, and compares the resulting source
fixture byte-for-byte. An explicit `--write` refreshes the fixture from the pinned
source only; changing an immutable edition's data still requires source review.
The Python regression test proves that an incorrect source edition cannot reach
the extractor or a fallback. No PDF, Python or Poppler is needed by consumers or
by normal offline Rust tests.

No unresolved extraction mismatch remains. Both extraction paths read the same
underlying PDF text layer, so agreement alone is not proof of correct rendering.
All 14 pages were rendered and visually inspected; the separate code inventory
and hand-read numerical fixtures cross-check that layer. Independent external
review of this final data/audit slice is required before closure. No independent
human cell-by-cell approval is claimed.

Category filtering does not belong in the catalog. Unknown table strings retain
`UnknownVersion`; exact-code lookup returns `None` for absent notation, including
unpublished combinations and variants. A known row does not establish engine
support, eligibility or correct execution. Footnotes and PCS factors above remain
data-only. Future editions require their own identity and fixtures and must
preserve this edition's values; the private initializer is specific to June 2.
