# Exact connection exclusions (#159)

The shared `ConnectionBuilder` uses Rust prefix choices for both program entry
and exact exclusions. A two-jump completion and its optional continuation are
separate choices. Selecting a prefix part edits from that position. Name/code
search filters the displayed next parts; it never changes the planning request.
The browser stores and sends the original backend whole key, including a
sequence's direct-step assumption. It does not implement jump grammar.

Excluding `1Lz+1Lo` leaves the component jumps and `1Lz+1Lo+1Lo` available.
Excluded wholes remain in a restore list when category or personal limits make
their components unavailable. The options response includes descriptions for
every supplied exclusion independently of the current permitted prefix choices.
A matching whole from another category restores its original saved key.

The existing App mirror and workspace serializer persist edits. The existing
search input signature invalidates/cancels work when an exclusion changes;
filtering, opening a builder and locale changes do not invalidate it. Ordinary
assessment still identifies imported excluded occurrences independently of
category findings and credited BV. Interactive exclusions now remove every
matching occurrence, including required ones, with undo. This supersedes the
earlier retained-conflict interaction. Search keeps invalid input separate from
a proved infeasible request. Restoration does not start calculation automatically.

The service validates `excludedConnections` in assessment, connection options
and search admission. A key must identify a whole compound in its original
registered context and preserve its exact transition assumption. Solo/spin keys,
mixed spin/jump strings and missing sequence assumptions produce a correlated
422 `invalid_connection_exclusion` on `excludedConnections`. The core's broader
exclusion API remains unchanged; this product field cannot create spin-level
holes. Validation ignores current personal limits so saved constraints survive.

## Verification

Executed on 2026-11-10 with Windows, Chrome 156 and native Debug services on
3165/3166:

- Public HTTP: 8 manual, 1 exclusion and 3 search tests passed. Cases cover pair versus
  longer whole, component availability, sequence identity, original-context
  preservation in all six real profiles and the independently registered
  Lutz-only policy, responsible occurrence conflicts, invalid field rejection
  and invalid-versus-infeasible search states. The synthetic policy correctly
  offers no Flip/Loop compound; its unavailable saved exclusion remains restorable.
- Four exclusion browser tests passed: actual add/remove/reload, unavailable
  limits/category preservation, pair/third separation, sequence keys, editing
  from a chosen part, translated names/conflicts, real 10.30 proof and retained
  conflict recovery, presentation-only filtering and cancellation of a held
  real creation response after an exclusion edit. Keyboard focus moves to the
  new restore action and returns to a remaining action or builder summary after
  removal. Three-language reflow was checked at 1280/390/320 CSS pixels, with
  restore targets at least 44 pixels high.
- All 24 browser tests passed after the final placement/focus changes. The first
  full run found that a separate top-level exclusion panel pushed the desktop
  sources below 720 pixels. Moving the controls into the existing workspace
  restored the desktop frame and passed the unchanged fit assertion. The final
  panel arrangement remains #162's work.
- `npm run build`, service clippy with warnings denied and `git diff --check`
  passed. No dependencies or core-library interfaces changed. The unchanged
  library suites were not repeated.

The worktree started at integration commit `8559ba5`; the latest integration
branch was checked before delivery. The final merge/check outcome is in the
delivery record.

Observed exclusion-to-assessed-conflict times were 318, 850, 837 and 842 ms, including
the 120 ms debounce and browser assertion. These are unthrottled observations;
they do not establish the parent's controlled timing gate. The initial synthetic
HTTP fixture incorrectly assumed Flip/Loop availability and omitted the
policy's ignored Waltz choice; the source-bound fixture was corrected. A focused
keyboard test exposed lost focus after exclusion and passed after focus moved
to the new restore action.

The independent 10.70/16-composition and exact 14-composition oracle belongs to
#158 and is shared parent regression evidence. Final filled-workspace layout,
the combined timing gate, screen-reader review and perspective walkthrough
remain parent/#162 acceptance work. Viewport automation is not physical-phone
or participant evidence; translations have not received native-language approval.

## Direct suggestion actions (#167)

Complete program cards show localized element names, whole codes, individual
credited base values and a program total. Repeated retained/added provenance is
secondary text. Both cards and comparison reuse the same element presentation
and named Exclude connection action. Solo jumps and spins have no such action.
The scope explanation is visible on desktop and phone; only the exact whole is
excluded, leaving its solo components and longer connections available.

The action submits the original service element key unchanged. Multiple supplied
assessment parts identify connections; the opaque key envelope preserves the
whole code and direct-step assumption. Only that whole suffix is compared across
categories, matching the existing service exclusion semantics. No connection
grammar, reconstruction from parts or display-text identity is introduced.

An exclusion removes all matching My program occurrences, regardless of required
status. Localized feedback names the whole and reports the removal count. Undo
removes that exclusion and restores removed IDs, keys, multiplicity, order and
required flags. Surviving original neighbours anchor restoration; later added
entries and edits to surviving entries are kept. The latest exclusion has one
in-memory undo, replaced by another exclusion and expired by central restoration,
adoption, category, ability or rotation changes. Reload retains the workspace,
not undo. Central restoration permits future use without re-inserting program
content. The optional builder and central list remain available independently.

Changed constraints dispose obsolete search work through the existing generation
guard. Old cards, totals and credited values remain visible and marked outdated;
adoption is disabled. Excluded connections remain marked, with a guarded
aria-disabled action retaining keyboard focus. Further exclusions can be collected
from these old cards. Update suggestions is the explicit recalculation action;
neither exclusion, undo nor restoration starts optimization. Pagination still
exposes every confirmed result with no new collection cap.

### Browser acceptance

`web/tests/suggestion-exclusions.spec.ts` exercises the real Silver fixture from
#164, complete collection, direct pair and sequence exclusion, original sequence
assumptions, no implicit search, refreshed exclusion absence across every page,
solo/longer negative controls, duplicates with required/interchangeable flags,
scoped undo after a later addition, repeated exclusions and central restoration.
A delayed real cursor response cannot publish obsolete results; its session is
disposed through the public HTTP route. Existing creation-race coverage remains.

The responsive scenarios capture cards, sequence action, undo, comparison and
central exclusions at 1280×800, 390×844 and 320×844 in
[the screenshot directory](evidence/167). They check named keyboard actions,
focus, 44-pixel targets, fixed-navigation clearance, three-language reflow and
central restoration after a category change. Viewport automation does not
establish physical-device or screen-reader certification.
