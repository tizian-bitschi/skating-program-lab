# Requirements

Senior Women and Men Free Skating 2026/27 are explicit profile selections under
Rule 612, with a separate recognition ruleset and shared technical policies.
They use maxima and required roles, not an exact completed-performance element
count. Plans, supplied recognition, qualified levels, contextual scoring and
deductions remain separate. The [Senior coverage matrix](rules/isu-senior-2026-27.md)
defines supported cases and uncertainty; marked advanced scoring and protocol
replay remain follow-up workstreams #78 and #80.

## Purpose and scope


Skaters and coaches need to plan and compare technical programs. The MVP targets singles Free Skating at Bronze, Silver, Gold and Masters for the 2026/27 ISU-supported International Adult competitions named in the [sources](rule-sources.md). This is a planning aid, not an official panel or judging system. Rules and scoring tables are explicitly versioned; an unknown version must not be treated as compliant or scored by a fallback version.

Artistic skating, pairs, dance, national variations, other seasons and categories, PCS, accounts, persistence, UI and HTTP API are outside the MVP. Later work may add versions without overwriting old results. Full Technical Panel inference, individual judges' panel simulation, automatic detailed spin-level recognition and expected-TES optimization are later work.


## Four distinct stages


1. **Plan:** an ordered program of intended elements and their intended positions, features and transitions. The validator checks local requirements and program-wide counts and dependencies against a selected ruleset.
2. **Execution:** observations of what was actually done, including movements, rotations, transitions and falls. Missing observations remain unknown; execution may differ from the plan.
3. **Recognition:** the panel-recognized element code, level, calls and any no-value or ignored outcome. In the MVP these are supplied by the user, not inferred from video or complete motion descriptions. The system checks supported input consistency and must not pretend to replace a panel decision.
4. **Scoring:** the recognized code and calls select base values and GOE treatment; element scores sum to TES. Separately reported program deductions include falls and time violations. A planned violation, a non-recognized element, a reduced base value, a zero value and a deduction are distinct outcomes.

Results must distinguish plan violations, recognition or input inconsistencies, scoring consequences and unknown or unsupported facts. Each finding identifies affected element(s), stable rule identifier, explanation, source and location. A single `is_valid` flag is insufficient; missing or unsupported information cannot produce a complete compliance claim. Where an official consequence or interpretation is unresolved, return unknown rather than inventing a result.


## Acceptance and quality


Correctness and source traceability take priority over convenience and speed. Use deterministic exact decimal or integer arithmetic and source-defined rounding. Version rules and tables with precise citations; unresolved questions block affected compliance or exact-score claims. Pure domain functions support independent tests and eventual native and WebAssembly use. Keep modules and dependencies small. No performance target is asserted before a representative workload exists.


**AC-07 — Versioning:** Identical input and explicit versions must produce identical results. Unknown versions must be rejected; adding a season must preserve earlier fixtures and sources. Incomplete or unsupported results must not claim complete compliance or an exact score.

See the [Adult rules](rules/isu-adult-2026-27.md), [scoring](specs/scoring.md) and [optimization](specs/optimization.md) specifications for detailed requirements and examples, and [implementation status](implementation-status.md) for current support.

## Extensible profile selection (#47)

Explicit profiles identify discipline, segment, season, federation/event scope,
category, policy version and exact source editions, with a separate SOV edition.
Supported typed policy can be registered without extending `AdultCategory`.
Legacy Adult APIs remain supported. A synthetic fixture exercises extension and
has no official category status. Pair/Short Program, feature/level recognition
and unknown mechanisms remain unsupported. See the accepted
[profile migration contract](adr/0002-rule-profiles-proposal.md).

The implemented bounded mixed-element scenario contract, finite domains, personal restrictions and proof limits are specified in [bounded program optimization](specs/bounded-program-optimizer.md).

## Product acceptance evidence

The [milestone #68 matrix](milestone-68.md) maps all four product workflows to
shared public entry points, independent reference cases, negative scenarios and
remaining coverage limits. A proven finite BV objective, complete declared
candidate generation, complete tie enumeration and full regulatory coverage are
four separate claims. The current model establishes the first three only in
explicit bounded conditional cases; regulatory coverage remains partial.
