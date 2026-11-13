# Alternative adoption and undo (#166)

Use this program replaces all interchangeable entries in My program with the
selected current suggestion. Required occurrences retain their exact element
keys, IDs and multiplicity, including two required occurrences of the same
element. Calculated additions get fresh IDs and remain interchangeable. The
replacement is explained beside each adoption button and in its accessible
description. Adoption opens My program, focuses its panel and announces the
replacement in a localized status region.

Undo adoption restores the immediately preceding entries, order, IDs, required
flags and fully-described/draft setting together. The editor then uses its
ordinary assessment and workspace-save path. Undo returns focus to Add an
element. Adopting A, then B, then undo restores A, including its calculated IDs.
Undo of an initial adoption restores the previous draft, including an empty
program. Language, panel navigation and comparison do not invalidate undo.

This is one in-memory action snapshot, not a history. Any later entry, order,
required flag, completeness, category, ability, exact exclusion or rotation
change expires the snapshot and explains why undo is unavailable. A new adoption
replaces the snapshot. Reload restores only the saved workspace; it does not
restore undo or search results. No timer limits the action.

The actions have different meanings:

| Action | Effect |
| --- | --- |
| Adopt a suggestion | Replace interchangeable entries; preserve required occurrences. |
| Remove an occurrence | Delete that occurrence from My program; do not forbid it in later suggestions. |
| Release a requirement | Keep that occurrence in My program but allow a later adoption to replace it. |
| Require an occurrence | Preserve that occurrence and its ID in the next calculation. |
| Exclude an exact connection | Restrict that whole connection in subsequent calculations; solo components and longer connections remain eligible. |

Changed required occurrences invalidate the existing calculation. Old cards
remain visible but cannot be adopted. Recalculation remains explicit. Reordering,
interchangeable edits and completeness changes preserve the existing composition
search semantics. The service, scoring and occurrence-identity contracts are
unchanged. Direct card exclusions and automatic removal after exclusion/ability
changes remain separate #164 slices; existing conflict reporting still applies.

## Acceptance evidence

`web/tests/adoption.spec.ts` runs against the real native planning service.
The shared Adult Silver fixture uses single jumps; USp Basic, SSp1, CSp Basic
and CCoSp1; other spins off; ChSq enabled; W and ChSp off; mixed directions off.
It does not assume a fixed alternative count or result ordering.

At 1280×800, 390×844 and 320×844, the tests adopt A and B around duplicate
required Loops, undo to A and undo the first adoption to the original draft.
They check occurrence identity, order, required flags, calculated content,
complete/draft assessment, explanation, keyboard activation, touch target sizes,
navigation clearance and horizontal overflow. Additional scenarios release and
add requirements, inspect the real next search request, reject stale adoption,
and expire undo after removal, reordering, completeness and constraint changes.
English, German and Russian phone scenarios use real touch events and keyboard
undo, checking localized accessible names, visible focus and focus recovery.

The first test failed on the previous UI at the missing Undo adoption action.
Representative screenshots are stored under `docs/evidence/166/`. Browser
emulation does not establish physical-device or screen-reader certification.

| Viewport | Before adoption | Adopted B | Undo to A | Replacement explanation |
| --- | --- | --- | --- | --- |
| 1280×800 | [Draft](evidence/166/original-1280.png) | [B](evidence/166/adopted-1280.png) | [A](evidence/166/restored-1280.png) | [Suggestions](evidence/166/suggestions-1280.png) |
| 390×844 | [Draft](evidence/166/original-390.png) | [B](evidence/166/adopted-390.png) | [A](evidence/166/restored-390.png) | [Suggestions](evidence/166/suggestions-390.png) |
| 320×844 | [Draft](evidence/166/original-320.png) | [B](evidence/166/adopted-320.png) | [A](evidence/166/restored-320.png) | [Suggestions](evidence/166/suggestions-320.png) |

Localized phone keyboard focus: [English](evidence/166/undo-en.png),
[German](evidence/166/undo-de.png), [Russian](evidence/166/undo-ru.png).
The targeted 14-scenario adoption suite and TypeScript/Vite build passed on
2026-11-13. The initial localized keyboard scenario exposed navigation focus
after adoption; focusing the program panel fixed the next Tab reaching Undo.
