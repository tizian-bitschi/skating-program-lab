# Rotation preference: design discussion

Status: product understanding confirmed by the user on 2026-11-08. The concrete
public API proposal is pending approval. See [implementation proposal](rotation-preference-proposal.md).

Prototype: branch `codex/prototype-rotation-preference`, commit `438e748`,
`src/planning_search/rotation-preference.prototype.html`.

## Settled choices

- Default to same-direction connections within each jump element. Mixed-direction
  connections require explicit opt-in; opting in permits rather than requires them.
- Apply the preference across all supported categories, including Senior.
- Apply it to selection, search and automatic completion.
- Preserve and calculate manual entries that conflict with the preference. Report
  a personal-preference conflict separately from competition-rule violations.
- Use one consistent default. The user confirmed there are no existing users;
  preserving the previous broad default for legacy calls has no product purpose.
- Do not constrain the absolute rotational direction or require separate solo
  jumps to share one direction.
- Automatically offer connections that support the selected direction preference.
  Do not introduce an "unresolved connection" product category. Establishing the
  source-backed transition classification is engineering work, not a user choice.
- If an immutable original conflicts with the preference, preserve it and report
  the conflict before automatic completion. The user must enable mixed-direction
  connections or change the original; manual BV calculation remains available.

## Confirmation

The user confirmed the complete product understanding. No further product
decisions are currently open. Public interface approval is separate.

## Implementation facts

The current code treats compound codes as assumed qualifying connections, without
deriving feet or rotational direction. Selection and maximum-BV search share the
planned jump enumerator. Completion consumes an explicit finite variant domain
and immutable originals. Manual BV assesses exact ordered selections.

Public request structs and function signatures will need a concrete design;
public API changes remain unapproved. The HTML prototype covers eight illustrative
connections and is not a complete transition model or an optimizer proof.
