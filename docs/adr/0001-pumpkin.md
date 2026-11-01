# ADR 0001: Pumpkin for a bounded native prototype

Decision recorded: 2026-11-01 (solver comparison PR #30).

Status: accepted for the bounded prototype; reassessment is required before a production commitment.

## Context

The optimizer needs a finite model, proof status and separate enumeration status. The [comparison](../solver-comparison.md) tested Pumpkin 0.5.0 and Z3 against the exhaustive Bronze reference. Eight assignments cannot support a performance or full-rule conclusion.

## Alternatives and decision

Keep the exhaustive reference as an oracle and use Pumpkin for the next bounded native implementation. Pumpkin has a native Rust API and matched the reference on this instance. Z3 also matched, but its Rust binding requires native libraries and adds deployment and cross-compilation costs. OR-Tools was not evaluated; the small comparison gave no basis for a broader conclusion. A solver framework is deferred until a concrete caller needs it.

## Consequences and reassessment

Pumpkin 0.5 requires a newer Rust toolchain and explicit model, termination and enumeration handling. It is optional and enabled by default. Proofs apply only to the encoded domain. Reassess toolchain requirements, WebAssembly builds, determinism, interruption semantics, larger workloads and external proof certificates before production use. The [optimization specification](../specs/optimization.md) records integration limits.
