# Skating Program Lab

An open-source foundation for planning technical figure skating programs. The first intended scope is singles free skating at Bronze, Silver, Gold and Masters in the 2026/27 ISU-supported International Adult Figure Skating Competitions.

The planned core will validate programs, simulate technical scores and find programs with maximum base value under rules and personal constraints. The Rust library currently models season, category, supported ruleset and scoring-table identities, and source references. It does not implement validation, scoring or optimization. Results from future versions must not be treated as official judging decisions.

## Development

Install stable Rust with `rustfmt` and `clippy`, then run:

```sh
cargo build
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Read [requirements](docs/requirements.md), [architecture](docs/architecture.md), [rule sources](docs/rule-sources.md), and [agent instructions](AGENTS.md) before contributing. Licensed under the [MIT License](LICENSE).
