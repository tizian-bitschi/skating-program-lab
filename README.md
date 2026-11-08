# Skating Program Lab

An open-source Rust library for planning technical figure skating programs. The initial scope is singles Free Skating in Adult Bronze, Silver, Gold and Masters under the 2026/27 ISU Adult rules. It is a planning and scoring-scenario tool, **not** a Technical Panel or an official judging system.

## Current capabilities

For theoretical program construction, use `assess_manual_bv` to calculate
base value from ordered SOV codes and check composition without supplying
spin or step feature evidence. See the [assumed-recognition API](docs/specs/assumed-recognition.md).
Prepare a `GoeScenarioPlan` from the selected codes to run
`preview_goe_scenario` with a uniform final GOE and individual overrides.
The [BV-first acceptance matrix](docs/milestone-89.md) covers Adult and Senior
manual BV, composition, full optimization and locked completion.

The [Senior Grand Prix integration audit](docs/milestone-75.md) maps the six
workstreams to parent acceptance evidence and a source-ordered eleven-slot
conditional completion scenario.

The [Senior Grand Prix benchmark report](docs/senior-grand-prix-benchmarks.md)
records offline source-derived domains, independent optimum/tie oracles and native
Debug/Release limits for full optimization and immutable completion.

The [official Grand Prix replay report](docs/grand-prix-2026-27-replay.md)
records exact TES parity for twelve source-verified programs and explicit
manual-assessment evidence gaps.

Senior numerical scoring also supports triple/quad calls and explicitly supplied
judge panels. [Protocol replay](docs/specs/senior-protocol-scoring.md) separates
printed official credit from timing verification and legacy hypothetical GOE.

- **Legality coverage:** `assess_program_legality` reports individual evaluated,
  conditional, missing, unsupported and nonapplicable rules, with source editions
  and event replacements. Draft omissions are indexed separately from final
  violations. See the [profile matrix and coverage contract](docs/specs/legality-coverage-proposal.md).

- **Locked completion:** `complete_program` inserts finite supported additions
  around immutable original entries, with explicit insertion boundaries, full
  ordered validation, provenance, BV subtotals and bounded equal-optimum proof.
  See the [completion contract](docs/specs/bounded-program-optimizer.md#immutable-ordered-completion-66).

- **Domain model:** Ordered programs with planned jumps, combinations, sequences and spin structures; versioned ruleset/table identities; and separately supplied execution observations, panel recognition and technical calls.
- **Validation:** Additive, profile-selected regular-spin plan checks with typed evidence and explicit family coverage, plus partial, source-linked jump checks for all four Adult categories, including jump eligibility, jump-element limits, combinations/sequences, Euler, repetitions and the Masters Axel requirement. Unresolved transitions and other missing information remain explicit; an empty finding list **does not** prove full program compliance.
- **Scoring:** Versioned base values and hypothetical final single-judge GOE for the twelve clean single and double jumps (1T through 2A) and supported technical-call variants, including cumulative error limits and supplied single-jump +REP. A limited set of recognized two- and three-jump combinations, supplied Euler/sequence units and component stars, all representable regular/V spins and levels, StSq, ChSq1, ChSp1 and selected zero-value outcomes are supported. Additive `*_with_execution` entry points accept separately supplied error evidence for verified interrupted-unit scenarios. Program scenarios can report TES and separate deductions where the required inputs and rules are supported; an optional second-half boundary is only a hypothetical planning assumption.
- **Candidate generation:** `generate_program_candidates` builds a deterministic finite supported domain from exact Adult profiles, jump selections, Basic-spin abbreviations, explicit evidence templates and personal filters. Its audit separates declared-domain completeness from solver proof; see the [contract](docs/specs/bounded-program-optimizer.md#finite-candidate-generation-65).
- **Optimization:** `optimize_program` handles finite ordered mixed-element Single Free Skating scenarios with explicit recognition assumptions, shared profile validation, personal exclusions and catalog scoring; see the [mixed-program contract](docs/specs/bounded-program-optimizer.md). `optimize_jumps` uses Pumpkin to maximize **unfactored jump base value** within an explicitly supplied finite jump-only domain for Bronze, Silver, Gold or Masters. It supports user exclusions, clean solo jumps and explicitly listed supported two-jump combinations. Results distinguish proven optimality, infeasibility, incomplete searches and completeness of equal-optimum enumeration. The earlier `optimize_bronze` and exhaustive reference remain limited 1Lz examples.

**Current limits:** Difficult-variation categories, some feature prerequisites, Euler consequences and full execution/panel recognition remain unmodeled. The mixed-program optimizer rejects unsupported variants and retains explicit proof limits. Optimization results are maxima only within their stated candidate domains, **not** official or global program maxima. For planned regular-spin checks, see the [supported-case matrix](docs/specs/regular-spin-validation.md). For exact coverage and limitations, see [implementation status](docs/implementation-status.md).

## Documentation

- Controlled optimizer integration acceptance: [#104 report](docs/milestone-104.md),
  with identical-request repeated benchmarks, native observations and remaining limits.

- Manual program entry point: `assess_manual_program` returns intended BV,
  independent optional GOE scenarios and indexed validation for one ordered plan.
  See the [assessment contract](docs/specs/manual-program-assessment-proposal.md)
  for missing-GOE, draft/completion and partial-coverage semantics.

- Product scope and stages: [requirements](docs/requirements.md).
- Module boundaries and data flow: [architecture](docs/architecture.md). Current support and tests: [implementation status](docs/implementation-status.md).
- Cross-workstream coverage, compatibility and remaining limits: [foundation milestone #44](docs/milestone-44.md).
- Four backend workflows, merged workstreams and acceptance evidence: [product milestone #63](docs/milestone-63.md).
- Domain work: [Adult rules](docs/rules/isu-adult-2026-27.md), [source editions and open questions](docs/rule-sources.md), and [source review](docs/rule-review.md).
- Feature work: [scoring](docs/specs/scoring.md) and [optimization](docs/specs/optimization.md), including the [category-aware jump optimizer](docs/specs/optimization.md#category-aware-jump-optimizer). Solver evidence: [Pumpkin ADR](docs/adr/0001-pumpkin.md) and [comparison](docs/solver-comparison.md).
- Coding agents: [AGENTS.md](AGENTS.md).

## Development

Install Rust with `rustup`; the repository selects the toolchain, `rustfmt` and `clippy`. Run:

```sh
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --no-default-features
```

The [solver comparison](docs/solver-comparison.md) has separate checks. Licensed under the [MIT License](LICENSE).
