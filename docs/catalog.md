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

At A, there are no production catalog rows and no lookup API yet. Scoring values
and behavior remain unchanged; catalog and scorer now use one private source
resolver on `ScoringTableId` for document/edition identity. The manifest test
checks every code against the separate source reference, metadata consistency,
unique identifiers, per-page and per-family counts, row ordinals, edition and
column counts. Later parity tests
compare production data with this separate manifest. Manifest updates require a
fresh source review; never regenerate expected coverage from production rows.

Pair data availability will not imply Pair recognition/scoring/validation support.
Category filtering does not belong in the catalog. Unknown table strings retain
`UnknownVersion`; exact-code lookup will return `None` for absent notation.
Future editions require their own identity and fixtures and must preserve this
edition's values.
