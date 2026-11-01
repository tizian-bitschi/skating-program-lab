# Skating Program Lab

An open-source Rust foundation for planning technical figure skating programs. The initial scope is singles Free Skating at Bronze, Silver, Gold and Masters in the 2026/27 ISU-supported International Adult competitions. The library provides partial validation and scoring and a [bounded Bronze optimizer](docs/specs/optimization.md#bounded-pumpkin-optimizer). An empty finding list does not establish complete compliance, and results are not official judging decisions.

## Documentation

- Product scope and stages: [requirements](docs/requirements.md).
- Module boundaries and data flow: [architecture](docs/architecture.md). Current support and tests: [implementation status](docs/implementation-status.md).
- Domain work: [Adult rules](docs/rules/isu-adult-2026-27.md), [source editions and open questions](docs/rule-sources.md), and [source review](docs/rule-review.md).
- Feature work: [scoring](docs/specs/scoring.md) and [optimization](docs/specs/optimization.md). Solver evidence: [Pumpkin ADR](docs/adr/0001-pumpkin.md) and [comparison](docs/solver-comparison.md).
- Coding agents: [AGENTS.md](AGENTS.md).

## Development

Install Rust with `rustup`; the repository selects the toolchain, `rustfmt` and `clippy`. Run:

```sh
cargo build
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The [solver comparison](docs/solver-comparison.md) has separate checks. Licensed under the [MIT License](LICENSE).
