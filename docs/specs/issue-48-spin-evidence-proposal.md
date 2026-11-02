# Issue 48: first-slice public API proposal

Status: additive interface and slice-1 delivery explicitly approved by the user on
2026-11-02. Implemented contract: [regular-spin validation](regular-spin-validation.md).

## Scope

Implement slice 1: typed regular-spin evidence and individual regular-spin checks.
Later slices retain program-wide feature reuse, step/choreographic recognition and
remaining jump mechanics. This slice does not close issue #48.

## Additive interface

- Keep `Program`, `RegularSpin`, `SpinStep`, `RuleProfile`, `ValidationResult` and
  all existing function signatures unchanged.
- Add `ProgramValidationEvidence`, containing evidence addressed by zero-based
  program element index. Reject duplicate indices and wrong element targets as
  input violations.
- Add `RegularSpinEvidence` with optional planned abbreviation/type, planned
  level, ordered typed feature attempts and optional separately supplied panel
  recognition. Preserve unknown versus explicitly empty feature lists.
- Add typed feature identifiers and attempt outcomes (unknown, unsuccessful,
  successful); preserve attempt order and position/foot targeting. These are
  caller assertions, not automatically recognized features.
- Reuse existing ordered `SpinStep` positions, revolutions and foot changes.
  Do not reinterpret free-text feature descriptions as confirmed features.
- Add an explicit, source-bound `RegularSpinPolicy` supplied alongside a selected
  `RuleProfile`; existing profile struct literals remain source compatible.
  Provide policies for the four built-in Adult profiles and permit an explicitly
  supplied synthetic policy using the same validation mechanisms. Never select
  Adult policy silently for an unknown profile.
- Add `validate_program_with_evidence` and a corresponding profile-selected
  entry point returning a new report that includes the existing findings plus
  explicit coverage by rule family and selected profile/source identities.
  Existing `validate_program` remains unchanged in this slice.

## Supported checks

Check category eligibility and maximum regular-spin counts, entry/type/foot-change
restrictions, known abbreviation differences, total and per-foot revolutions,
and basic positions held for two revolutions. Distinguish prohibited plans from
conditional no-value/V consequences; never manufacture a panel call. Missing
facts produce Unknown, mechanisms outside this slice produce Unsupported.
Do not infer a recognized level from a supplied plan or feature count.

Minimum sources: Adult announcement updated 2026-07-16 pp. 8–11;
Communication 2788 updated 2026-05-19, relevant spin provisions; applicable
Single Skating Technical Panel Handbook. Verify source wording before coding.

## Validation and review

Independent neighboring valid/invalid/unknown fixtures for all four categories,
foot changes and revolution boundaries; legacy jump regression tests; synthetic
policy tests; cargo fmt --check; cargo clippy --all-targets -- -D warnings;
cargo test; cargo test --no-default-features. One draft PR for this bounded slice,
external GPT-6 / Sehr Hoch review in Chrome. No issue closure for a partial slice.

## Approval requested

Approve the additive public types and entry points described above, plus delivery
of slice 1 in one PR while keeping #48 open for the remaining slices. Exact names
may follow existing repository conventions; no breaking public changes authorized.
