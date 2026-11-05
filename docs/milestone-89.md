# Milestone #89 — BV-first planning acceptance

These fixtures use the Adult and Senior 2026/27 Free Skating profiles and
the 2 June 2026 edition of ISU Communication 2786. The scenarios select
published codes and levels as inputs to planning.

| Workflow | Adult evidence | Senior evidence |
| --- | --- | --- |
| Manual BV | `tests/manual_bv.rs::gold_spin_draft_has_base_value_before_a_goe_request` checks the two-spin draft. The same suite covers indexed selection failures and partial BV. `tests/element_selection.rs::manual_base_at_each_level_boundary_uses_the_exact_selection_source` checks category caps against the catalog. | `tests/assumed_recognition.rs::senior_levels_and_fixed_codes_are_edition_bound` checks Basic, levels 1–4, fixed choreography and table identity. `senior_manual_total_uses_eighty_percent_for_second_advanced_solo` checks contextual Rule 612 credit. |
| Composition | `tests/assumed_recognition.rs` covers structural violations, level caps and the separate evidence-based qualification checks. `tests/element_selection.rs` covers category-safe types and levels. | `tests/assumed_recognition.rs` covers Senior level and vocabulary findings. `tests/senior_profiles.rs` covers the detailed Senior profile and evidence path. |
| Full optimization | `tests/program_optimizer.rs::optimize_gold_sov_selections_without_spin_or_step_proof` compares the optimum and every ordered tie against an exhaustive fixture with literal SOV coefficients. | `tests/program_optimizer.rs::assumed_senior_full_program_matches_manual_bv` compares both tied flying-spin choices and their orders against an exhaustive fixture. |
| Locked completion | `tests/program_completion.rs::gold_code_domain_completion_matches_manual_bv_and_order_oracle` checks every optimal placement, the original entries and insertion boundaries. | `tests/program_completion.rs::assumed_senior_completion_keeps_distinct_equal_bv_choices` checks fixed anchor identities, exact BV and both equally valuable additions. `assumed_completion_rejects_senior_repeat_credit_across_locked_original_and_addition` checks the unsupported repetition domain. |

The optional GOE preview is verified in `tests/goe_scenario.rs`. The Adult
Gold fixture checks five final grades against the published columns and covers
overrides, missing grades, calls, falls and distinct scoring outcomes. The
Senior fixture verifies the GOE column, Rule 612 credit and a second-half
assumption for the second solo jump.

The catalog and selection audit identify the source edition and the caller's
finite domain. Legality coverage identifies the structural rules evaluated
and evidence checks that require detailed inputs. Solver status records the
objective proof within the supported domain. Enumeration status separately
records whether all equally valuable results were returned. Scenario coverage
applies to the supplied GOE values and scoring consequences for a selected
program. Technical recognition, actual performance and production-scale
interactive latency are outside these fixtures.

Regression checks include the source-backed Adult/Senior fixtures, the
twelve-program exact TES replay in `tests/official_replay.rs`, detailed spin
validation, full optimization and locked completion. Run `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` and
`cargo test --no-default-features`. Solver tests require the `optimizer`
feature. Issues #56 and #58 remain outside this milestone.
