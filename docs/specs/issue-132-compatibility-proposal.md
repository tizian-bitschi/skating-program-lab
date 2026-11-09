# Issue #132 compatibility proposal

Status: **Pending user approval**. This proposal records two separate decisions;
it grants neither approval and does not amend the accepted ADR or agent rules.

## Public enum additions

[AGENTS.md](../../AGENTS.md) requires explicit approval before changing public
interfaces. The integrated issue #132 work adds these cases to existing public
Rust enums:

| Enum | Added case | Caller migration |
| --- | --- | --- |
| `PlanningExclusion` | `ExactElement(String)` | Handle an exact whole selectable code, preserving compound identity and leaving its components available elsewhere. |
| `PersonalLimitConflictKind` | `StepLevel { requested: Level, maximum: Level }` | Handle a personal step-level ceiling conflict independently of competition rules. |
| `JumpKind` | `Waltz` | Handle a planned Waltz intention separately from clean Axel; its recognition and category treatment remain explicit. |
| `RuleMechanism` | `AdultWaltzPlanV1(WaltzPolicy)` | Handle source-bound Adult eligibility treatment selected by `WaltzPolicy`. |

These additions break downstream exhaustive matches even though existing enum
cases and public struct layouts remain intact. Consumers must add the named
arms (or an appropriate fallback) and recompile. Existing stored variants keep
their meaning; no storage format migration is introduced. The exact additions
above require explicit user approval before integration is accepted.

## Narrow profile-version exception

[ADR 0002](../adr/0002-rule-profiles-proposal.md) says: “A changed
policy or source edition requires a new profile version”. The
[architecture](../architecture.md) also requires versioned rules without mutating
older versions. Current implementation corrections retain the built-in
`isu-adult-2026-27-<category>` IDs and version `2026-07-16` while correcting
Silver's `max_choreographic_spins` and Adult eligibility mechanisms.

Approve a narrowly scoped exception for implementation corrections against the
same authoritative editions: Silver's permitted second spin of any nature
(including ChSp), the category-specific bare-W policy, and the inherited jump-kind
cap shared by W and its intended Axel identity. The corrected Silver maxima
are 1480 unrestricted/restricted-only and 1470 required-only/combined hundredths,
replacing 1430 and 1420. The July 16 Adult announcement and the selected SOV
edition remain unchanged; see [acceptance evidence](../milestone-132.md) and
[Waltz contract](waltz-planning.md).

Recommendation: retain these source-edition IDs for these corrections and record
the corrected outcomes in acceptance evidence. Consumers comparing old calculated
results must recalculate them; the same ID no longer reproduces the prior buggy
result. An actual future change to rule policy, interpretation scope or source
edition still requires a new profile version under ADR 0002. This exception does
not authorize other same-ID policy changes.

Explicit user approval is required for **both** the enum additions and this
profile-version exception. Until both decisions are recorded, compatibility
approval remains open regardless of passing tests. No ADR or AGENTS change is
included in this proposal.

