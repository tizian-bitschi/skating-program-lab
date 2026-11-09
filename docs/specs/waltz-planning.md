# Planned Waltz intentions

`W` declares a standalone Waltz intention under the Adult announcement:
Bronze ignores it; Silver and Gold call it as a downgraded single Axel.
`W(1A<<)` explicitly declares the intended downgraded-single-Axel classification
under the general jump rules, including Masters and Senior profiles. Bare `W`
there lacks a selected classification and is an unsupported input code, not a
claim that the movement is prohibited.

The Adult announcement dated 16 July 2026, printed pp. 9–11, supplies the
Silver/Gold and Bronze treatments. The Technical Panel Handbook dated 15 July
2026, pp. 28–30, distinguishes intended Waltz preparation from a called jump and
applies downgrade rules to the intended listed jump. Communication 2786 dated
2 June 2026, p. 3, publishes `1A<<` at 0.00.

The plan keeps `JumpKind::Waltz` separate from `JumpKind::Axel`; its `rotations`
field is `Some(1)`, identifying the single Waltz class, not one full physical
revolution. Recognition explicitly carries a downgraded 1A call. The immutable
catalog contains no invented W row. `selectable_elements` and its catalog-row
contract remain unchanged. `selectable_jump_elements` exposes the planned Waltz
codes; use these exact codes for requirements and whole-element exclusions.
Manual assessment points valuation provenance to the published `1A<<` row.

| Intention | Jump boxes | Repetitions | Mandatory Axel | BV |
| --- | --- | --- | --- | --- |
| Bronze `W` | None | Ignored | Does not satisfy | 0.00 |
| Silver/Gold `W` | One | Shares intended 1A stock | Satisfies if required by selected policy | 0.00 |
| Explicit `W(1A<<)` | One | Shares intended 1A stock | Satisfies | 0.00 |

Called intentions can occupy permitted compound positions. The selected
combination/sequence limits, direct-step assumption and rotation preference
apply, and the Waltz component contributes zero to the sum. A sequence can
explicitly step into the intended downgraded Axel. Bronze W is an ignored
standalone movement and is not generated as a recognized compound component.

Preparation-only Waltz movements remain transition/evidence information. They
are outside the technical-element repertoire; these codes never infer from
footwork whether a Waltz is preparation. No decorated technical-code aliases
are generated for preparation movements.

`RuleMechanism::AdultWaltzPlanV1(WaltzPolicy)` binds the category treatment to the
Eligibility source. Registered profiles can reuse it without a category-name
branch. General jump mechanisms admit `W(1A<<)` only when single Axel is allowed.
Personal listed-rotation ceilings leave this single-class intention available;
semantic Axel/1A exclusions also inspect its called identity. Exact whole-code
exclusions distinguish W from clean 1A and leave other compounds available.

Ignored Bronze W has unbounded optional multiplicity. Search proves the BV
objective without this neutral movement, restores every required occurrence
with its original input identity, and reassesses five compositions differing
in optional W multiplicity. Its alternatives are `LimitReached`, never
`Exhausted`. Excluding W restores finite-domain alternative exhaustion. This
also holds when the proved maximum is zero. Cancellation retains separate
maximum-proof and alternative-collection semantics.

## Verification for #136

Focused default-feature checks passed for Waltz planning (7), planning search
(23), registered categories (2), exact element selection (6), alternative
identity/cancellation (5), bound pruning (5), personal limits (13), step limits
(4), library/grammar and listed-domain regression tests. `cargo build`,
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` passed.
The full `--no-default-features` suite passed before final catalog-adapter
cleanup; the final focused minimal Waltz suite (4) also passed.

A broad default-feature run exposed zero-weight affine-view panics and a Senior
bound that duplicated the shared W/1A stock. Both were fixed and affected
focused checks passed. The run was stopped before the slow unrestricted Senior
and ordered optimizer tests finished; it is not a successful full-suite result.
Logs remain under `target/waltz-checks`. The final full integration run and
native Release comparison belong to parent #132 acceptance.

The expanded Masters repertoire needs a larger cooperative alternative-poll
budget in the existing regression fixture: 14,000 polls yielded four results;
20,000 yielded five. This is a disclosed enumeration-cost increase, not a new
wall-clock performance guarantee. The separate clean-1Lz solver-comparison
prototype is unchanged and was not rerun.
