# Suggestion comparison (#169)

Comparison is a subordinate view of Suggestions. Select any number of confirmed
programs across result pages, then open comparison. Both lists render twenty
programs per page without limiting selection or collected results. Returning
restores the result page, selection, reading position and comparison trigger
focus. Switching planning panels retains the open view and search session.
Desktop comparison uses the full workspace width and the document scroll.

Differences are whole-element multiset differences. Each element's opaque service
key includes compound order and sequence assumptions. The minimum occurrence
count across selected programs is identical content; remaining occurrences are
differences. Reordering whole elements changes neither grouping nor composition.
Identical content expands separately for each program, preserving every localized
name, credited value, required/added provenance and direct exclusion action.
There is no client-side jump grammar or scoring calculation.

The shared result-element renderer emits the same whole-connection exclusion as
cards. The workspace removes every matching program occurrence, offers undo and
invalidates the old search. Comparison values and entries remain visibly outdated,
adoption is disabled, and additional exclusions remain available. Only explicit
Update suggestions starts a replacement optimization.

## Acceptance evidence

`web/tests/comparison.spec.ts` exercises the real Adult Silver service: selection
across pages, differences and expanded identical content, pair and sequence
exclusion, undo, stale adoption, explicit update, panel switches and phone return.
The existing duplicate-occurrence exclusion test now invokes the action from
comparison and verifies exact restoration while preserving later program edits.
The large-result regression retains live-append focus and selection checks.

`comparison-identity.spec.ts` is explicitly synthetic presentation evidence. It
supplies permutations that the optimizer normally deduplicates, checks whole
element multiplicity and internal compound order, and selects 24 programs across
pages. It makes no optimization or scoring claim.

Screenshots cover 1280×800, 390×844 and 320×844 in English, German and Russian,
including expanded long sequence names and keyboard focus. Browser assertions
check horizontal overflow, target size and bottom-navigation clearance. This
does not establish screen-reader conformance or a sport-rule audit.

Representative evidence: [desktop](evidence/169/comparison-1280-en.png),
[phone](evidence/169/comparison-390-de.png),
[narrow Russian](evidence/169/comparison-320-ru.png).
The previous repeated full-program comparison is shown in
[the earlier comparison evidence](evidence/167/comparison-1280.png).

This supersedes the old full-content comparison presentation in C2 acceptance;
the service identity, lifecycle and exact-exclusion contracts remain unchanged.
