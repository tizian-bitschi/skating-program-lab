# Issue #132 compatibility decisions

Status: **Accepted by the user on 2026-11-09** in the requesting chat. The user
explicitly approved both the enum additions and the narrowly scoped profile-version
exception below. ADR 0002 records the exception.

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
above were explicitly approved by the user on 2026-11-09.

## Accepted profile-version exception

[ADR 0002](../adr/0002-rule-profiles-proposal.md) says: “A changed
policy or source edition requires a new profile version”. The
[architecture](../architecture.md) also requires versioned rules without mutating
older versions. Current implementation corrections retain the built-in
`isu-adult-2026-27-<category>` IDs and version `2026-07-16` while correcting
Silver's `max_choreographic_spins` and Adult eligibility mechanisms.

The user approved a narrowly scoped exception for implementation corrections against the
same authoritative editions: Silver's permitted second spin of any nature
(including ChSp), the category-specific bare-W policy, and the inherited jump-kind
cap shared by W and its intended Axel identity. The corrected Silver maxima
are 1480 unrestricted/restricted-only and 1470 required-only/combined hundredths,
replacing 1430 and 1420. The July 16 Adult announcement and the selected SOV
edition remain unchanged; see [acceptance evidence](../milestone-132.md) and
[Waltz contract](waltz-planning.md).

Retain these source-edition IDs for these corrections and record
the corrected outcomes in acceptance evidence. Consumers comparing old calculated
results must recalculate them; the same ID no longer reproduces the prior buggy
result. An actual future change to rule policy, interpretation scope or source
edition still requires a new profile version under ADR 0002. This exception does
not authorize other same-ID policy changes.

Both compatibility decisions are approved. The exact scope above and the
[ADR 0002 exception](../adr/0002-rule-profiles-proposal.md#approved-correction-exception-132)
record the decision alongside the acceptance evidence.

