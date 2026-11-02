# Skating Program Lab

An open-source Rust library for planning technical figure skating programs. The initial scope is singles Free Skating in Adult Bronze, Silver, Gold and Masters under the 2026/27 ISU Adult rules. It is a planning and scoring-scenario tool, **not** a Technical Panel or an official judging system.

## Current capabilities

- **Domain model:** Ordered programs with planned jumps, combinations, sequences and spin structures; versioned ruleset/table identities; and separately supplied execution observations, panel recognition and technical calls.
- **Validation:** Partial, source-linked jump checks for all four Adult categories, including jump eligibility, jump-element limits, combinations/sequences, Euler, repetitions and the Masters Axel requirement. Unresolved transitions and other missing information remain explicit; an empty finding list **does not** prove full program compliance.
- **Scoring:** Versioned base values and hypothetical final single-judge GOE for the twelve clean single and double jumps (1T through 2A) and supported technical-call variants, including cumulative error limits and supplied single-jump +REP. A limited set of recognized two- and three-jump combinations with later toe loops or loops, all representable regular/V spins and levels, StSq, ChSq1, ChSp1 and selected zero-value outcomes are supported. Program scenarios can report TES and separate deductions where the required inputs and rules are supported; an optional second-half boundary is only a hypothetical planning assumption.
- **Optimization:** `optimize_jumps` uses Pumpkin to maximize **unfactored jump base value** within an explicitly supplied finite jump-only domain for Bronze, Silver, Gold or Masters. It supports user exclusions, clean solo jumps and explicitly listed supported two-jump combinations. Results distinguish proven optimality, infeasibility, incomplete searches and completeness of equal-optimum enumeration. The earlier `optimize_bronze` and exhaustive reference remain limited 1Lz examples.

**Current limits:** Spin feature/level validation, step and choreographic-sequence recognition rules, many combination/sequence and technical-call consequences, and complete Free Skating optimization are not implemented. Optimization results are maxima only within their stated candidate domains, **not** official or global program maxima. For exact coverage and limitations, see [implementation status](docs/implementation-status.md).

## Documentation

- Product scope and stages: [requirements](docs/requirements.md).
- Module boundaries and data flow: [architecture](docs/architecture.md). Current support and tests: [implementation status](docs/implementation-status.md).
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
