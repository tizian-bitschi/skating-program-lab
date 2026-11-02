# Issue 45: catalog API and delivery proposal

Status: public API and three-PR delivery approved by the user on November 2, 2026. See [catalog reference](catalog.md) for implementation status.

## Public API

Add a `catalog` module alongside scoring. Preserve all existing scoring APIs and
`ScoringTableId::Isu2786June2_2026`.

Proposed entry point:

```rust
pub fn catalog_row(
    table: ScoringTableId,
    notation: &str,
) -> Option<&'static CatalogRow>;
```

Notation is the exact published code, including level and technical variant.
Unknown codes return `None`. Unknown editions remain `UnknownVersion` through
the existing table-ID parser. No fallback, category filtering or recognition
inference occurs.

`CatalogRow` exposes immutable exact notation, discipline/family, level where
published, BASE in hundredths, all eleven signed GOE columns in hundredths,
source document/edition/printed page/row, and references to published notes.
Published special factors and data-only notes remain catalog metadata. A row's
presence does not assert that recognition or scoring behavior is implemented.
Existing engine support remains determined by existing scoring APIs.

Use typed embedded Rust data and integer values, with no runtime PDF parsing or
new dependency. Keep the independently source-reviewed coverage manifest in a
separate test fixture. Production data must not generate its own expected fixture.

## Delivery

Follow the issue's explicit small-PR boundaries:

1. A: visually verify the June 2 source, inventory every section and row in the
   independent manifest, introduce the approved typed data contract, and document
   unresolved source cells. Keep this PR independently buildable.
2. B: populate jumps, spins, steps and choreographic sections, with independent
   fixtures and all-column checks. Migrate existing supported rows to the shared
   data without expanding engine behavior.
3. C: populate Pair sections and finish the all-section completeness audit,
   footnote documentation and full validation.

Each PR targets `main`, gets its own external GPT-6 / Sehr Hoch review in Chrome,
and merges only after APPROVE and required checks. Close issue #45 only after C
satisfies all acceptance criteria. PRs A and B reference the issue without closing it.
This intentionally needs authorization to adapt the invoked skill's single-PR
and immediate-issue-closure steps to the issue's explicit multi-PR requirement.

## Validation

Each applicable slice runs `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, and
`cargo test --no-default-features`. Final completeness checks reject missing or
duplicate identifiers, missing GOE cells and source/version mismatches. Independent
source fixtures cover every family, multiple levels, technical calls and notes;
existing twelve-jump and CSSp1/ChSp1 behavior must remain unchanged.

