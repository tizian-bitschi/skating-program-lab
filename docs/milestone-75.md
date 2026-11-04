# Senior Grand Prix integration milestone (#75)

Acceptance baseline: `b8f9825` (all six child PRs merged). This audit adds
one source-ordered eleven-slot regression and connects the parent criteria to
existing independent evidence. It changes no public API, rule, solver or CI.

## Workstreams and acceptance evidence

| Workstream | Merged PR | Evidence |
| --- | --- | --- |
| #76 Protocol dataset | [#82](https://github.com/tizian-bitschi/skating-program-lab/pull/82) | [Source manifest and transcription review](fixtures/grand-prix-2026-27.md); `tests/grand_prix_fixtures.rs` |
| #77 Senior profiles | [#83](https://github.com/tizian-bitschi/skating-program-lab/pull/83) | [Rule 612 coverage and editions](rules/isu-senior-2026-27.md); `tests/senior_profiles.rs` |
| #78 Numerical scoring | [#84](https://github.com/tizian-bitschi/skating-program-lab/pull/84) | [Panel-scoring contract](specs/senior-protocol-scoring.md); `tests/panel_scoring.rs` |
| #79 Eleven-slot capacity | [#85](https://github.com/tizian-bitschi/skating-program-lab/pull/85) | [Engineering audit](specs/bounded-program-optimizer.md#eleven-slot-engineering-audit-79); `tests/senior_profiles.rs` |
| #80 Exact TES replay | [#86](https://github.com/tizian-bitschi/skating-program-lab/pull/86) | [Twelve program results and discrepancy contract](grand-prix-2026-27-replay.md); `tests/official_replay.rs` |
| #81 Performance baseline | [#87](https://github.com/tizian-bitschi/skating-program-lab/pull/87) | [Measured domains, runner revisions and limits](senior-grand-prix-benchmarks.md); `tests/senior_benchmarks.rs` |

| Parent criterion | Checked result and boundary |
| --- | --- |
| Official Senior Women/Men 2026/27 FS, distinct from Adult | Exact Senior identities select shared source-bound Rule 612 policies; unknown identities/segments do not fall back to Adult. `exact_identities_and_shared_technical_definition` and `custom_profiles_and_event_bindings_do_not_fall_back_to_adult` pin this contract. Four Adult profiles remain separate. |
| Twelve independently checked programs across four groups | Three programs per event/category, 132 ordered elements. `grand_prix_fixtures` compares every factual field with the separately reviewed transcription and tests corruptions. Exact PDF URLs, editions, SHA-256 and source locations remain in `sources.tsv`. This is an agent source cross-check, not human certification. |
| Exact BV, credited BASE, panel GOE and TES for supported reports | `all_twelve_official_programs_have_independently_verified_exact_tes` matches all twelve TES totals to hundredths. Independent SOV extraction and transcription supply expected values. Missing/invalid marks, unknown recognition, altered grades and row order prevent exact status. PCS and deductions stay outside TES. |
| Eleven-slot full and completion contracts | #79 covers capacity, numeric bounds, provenance, ties and rescoring; zero-deadline full requests establish capacity, not successful search. The new source-order test below successfully completes and reassesses an eleven-slot restricted conditional plan. |
| Real-program performance and independent correctness | #81 records 192 native observations, fixed source-derived domains, budgets, wall time, proposals, omissions, status and ties. Independent small-domain oracles and medium completion values pass. The #68 baseline remains intact; differing domains establish no speedup. Broad full/large completion runs return no incumbent. |
| Editions, compatibility, both feature configurations | June 2 Communication 2786 remains the immutable 625-row table. Existing catalog, Adult, scoring and product API suites run with default and disabled optimizer features. Source reviews include the October Q&A; no 2025/26 fallback is introduced. |

## Source-ordered eleven-slot scenario

`senior_benchmarks::source_order_eleven_slot_conditional_plan_is_completed_and_rescored`
uses Ami NAKAI's France Women rank-1 report, page 1, all eleven source ordinals:

`3A, 3Lz+3T, 3F!, FSSp4, ChSq1, 3Lo+2A+2A+SEQ, 3Lz, 3Lo, CCoSp4, StSq4, ChSp1`.

The test preserves the jump kinds, rotations, compound structure and complete
family order. It supplies hypothetical clean recognition (`3F!` becomes `3F`),
direct jump transitions and the existing Basic spin/step movement templates.
Those assumptions are explicit; published levels, GOE, second-half credit and
movement evidence are not imported into the planning objective. Each original
retains a PDF URL, digest and row location labelled as a conditional abstraction.
The unchanged raw protocol remains in the dataset and exact numerical replay.

All eleven entries are locked, additions are disabled and the domain has one
ordered candidate. Completion proves only this singleton domain, preserves
entries/evidence/recognition and revalidates/rescores the result. Independent
June 2 coefficients give jump BV 45.70 and Basic role BV 12.60, total **58.30**.
No official TES or global optimum follows. The separate #79 full-search capacity
and #81 broad-search measurements retain their incomplete outcomes.

## Validation and remaining limits

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` and `cargo test --no-default-features`. Fixture verification is part
of the test suites. The benchmark report pins the historical runner and lock;
`pwsh -NoProfile -File benchmarks/senior-grand-prix/test-runner.ps1` checks its
revision/cleanliness guards. New measurements must use the pinned reproduction
instructions; this audit does not relabel historical CSVs as a fresh run.

Numerical protocol parity does not establish independent panel recognition,
timing or full plan legality. Missing motion/feature evidence remains explicit.
Senior repeated high solos can produce `InconsistentAssessment` in completion
because manual 80% credit and the unfactored optimizer objective differ; #81
excludes those additions and tests the error. No pruning improvement, broad
interactive success or global optimum is established. #56 and #58 remain open.
Short Programs, Junior, Pair, Ice Dance, PCS/TSS reconstruction and UI/HTTP remain
outside this milestone.
